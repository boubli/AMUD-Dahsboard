package services

import (
	"context"
	"log/slog"
	"net/http"
	"strings"
	"sync"
	"time"

	"github.com/boubli/AMUD-Dashboard/internal/database"
)

var AppStatuses sync.Map // Map[int64]bool

func StartPingEngine() {
	ticker := time.NewTicker(30 * time.Second)
	slog.Info("Starting background asynchronous network ping engine (30s interval)")
	// Run immediately on startup
	go pingAll()

	go func() {
		for range ticker.C {
			pingAll()
		}
	}()
}

func pingAll() {
	if database.DB == nil {
		return
	}

	slog.Debug("Starting concurrent network ping scan across all targets")

	rows, err := database.DB.Query("SELECT id, url FROM apps")
	if err != nil {
		slog.Error("Ping scan query failed", "error", err)
		return
	}
	defer rows.Close()

	var wg sync.WaitGroup
	client := &http.Client{
		Timeout: 2 * time.Second,
		CheckRedirect: func(req *http.Request, via []*http.Request) error {
			return http.ErrUseLastResponse
		},
	}

	var count int
	for rows.Next() {
		var id int64
		var url string
		if err := rows.Scan(&id, &url); err != nil {
			continue
		}

		count++
		wg.Add(1)
		go func(appID int64, appURL string) {
			defer wg.Done()
			online := checkURL(client, appURL)
			AppStatuses.Store(appID, online)
			slog.Debug("Ping check completed", "app_id", appID, "url", appURL, "online", online)
		}(id, url)
	}
	wg.Wait()
	slog.Debug("Ping scan sequence finalized", "completed_count", count)
}

func checkURL(client *http.Client, targetURL string) bool {
	targetURL = strings.TrimSpace(targetURL)
	if targetURL == "" {
		return false
	}

	if !strings.HasPrefix(targetURL, "http://") && !strings.HasPrefix(targetURL, "https://") {
		targetURL = "http://" + targetURL
	}

	req, err := http.NewRequestWithContext(context.Background(), "HEAD", targetURL, nil)
	if err != nil {
		return false
	}
	req.Header.Set("User-Agent", "AMUD-PingEngine/1.0")

	resp, err := client.Do(req)
	if err == nil {
		resp.Body.Close()
		return true
	}

	req, err = http.NewRequestWithContext(context.Background(), "GET", targetURL, nil)
	if err != nil {
		return false
	}
	req.Header.Set("User-Agent", "AMUD-PingEngine/1.0")

	resp, err = client.Do(req)
	if err == nil {
		resp.Body.Close()
		return true
	}

	return false
}
