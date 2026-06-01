package discovery

import (
	"context"
	"database/sql"
	"encoding/json"
	"fmt"
	"log"
	"net"
	"net/http"
	"strconv"
	"strings"
	"time"

	"github.com/boubli/AMUD-Dashboard/internal/database"
)

type ContainerInfo struct {
	ID    string   `json:"Id"`
	Names []string `json:"Names"`
	Image string   `json:"Image"`
	State string   `json:"State"`
	Ports []struct {
		IP          string `json:"IP"`
		PrivatePort int    `json:"PrivatePort"`
		PublicPort  int    `json:"PublicPort"`
		Type        string `json:"Type"`
	} `json:"Ports"`
	Labels map[string]string `json:"Labels"`
}

func StartAutoDiscovery() {
	ticker := time.NewTicker(60 * time.Second)
	// Run initially on startup
	go scanDockerSocket()

	go func() {
		for range ticker.C {
			scanDockerSocket()
		}
	}()
}

func scanDockerSocket() {
	if database.DB == nil {
		return
	}

	client := &http.Client{
		Timeout: 5 * time.Second,
		Transport: &http.Transport{
			DialContext: func(ctx context.Context, network, addr string) (net.Conn, error) {
				return net.Dial("unix", "/var/run/docker.sock")
			},
		},
	}

	// Fetch only running containers from Docker socket
	resp, err := client.Get("http://localhost/containers/json")
	if err != nil {
		// Docker socket not accessible (e.g. host is non-linux/local dev)
		return
	}
	defer resp.Body.Close()

	var containers []ContainerInfo
	if err := json.NewDecoder(resp.Body).Decode(&containers); err != nil {
		return
	}

	for _, c := range containers {
		// Check for auto-discovery label or if container has published public ports
		enableVal, enableLabelExists := c.Labels["amud.enable"]
		if (enableLabelExists && enableVal == "true") || (!enableLabelExists && len(c.Ports) > 0) {
			name := ""
			if len(c.Names) > 0 {
				name = c.Names[0]
				name = strings.TrimPrefix(name, "/")
			} else {
				continue
			}

			// Determine public port mappings
			port := 80
			for _, p := range c.Ports {
				if p.PublicPort != 0 {
					port = p.PublicPort
					break
				}
			}

			// Clean name representation
			name = strings.Title(strings.ReplaceAll(name, "-", " "))

			// Check if container already registered
			var dbID int64
			err := database.DB.QueryRow("SELECT id FROM apps WHERE name = ?", name).Scan(&dbID)
			if err == sql.ErrNoRows {
				targetURL := "http://localhost:" + strconv.Itoa(port)
				description := fmt.Sprintf("Auto-discovered container running image: %s", c.Image)
				icon := "container"
				category := "General"
				nodeTag := "Local"

				// Check labels for customizable attributes
				if customIcon, exists := c.Labels["amud.icon"]; exists {
					icon = customIcon
				}
				if customDesc, exists := c.Labels["amud.description"]; exists {
					description = customDesc
				}
				if customCat, exists := c.Labels["amud.category"]; exists {
					category = customCat
				}
				if customTag, exists := c.Labels["amud.node_tag"]; exists {
					nodeTag = customTag
				}

				_, err = database.DB.Exec(
					"INSERT INTO apps (name, url, icon, description, category, node_tag) VALUES (?, ?, ?, ?, ?, ?)",
					name, targetURL, icon, description, category, nodeTag,
				)
				if err != nil {
					log.Printf("auto-discovery: failed to register %s: %v", name, err)
				} else {
					log.Printf("auto-discovery: registered new application %s", name)
				}
			}
		}
	}
}
