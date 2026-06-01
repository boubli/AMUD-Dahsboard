package handlers

import (
	"context"
	"database/sql"
	"fmt"
	"html/template"
	"net"
	"net/http"
	"strconv"
	"strings"
	"time"

	"github.com/boubli/AMUD-Dashboard/internal/auth"
	"github.com/boubli/AMUD-Dashboard/internal/database"
	"github.com/boubli/AMUD-Dashboard/internal/models"
	"github.com/boubli/AMUD-Dashboard/internal/services"
)

var Templates *template.Template

type DashboardSettings struct {
	AppName         string
	Tagline         string
	SearchEnabled   bool
	ShowGreeting    bool
	ShowClock       bool
	BackgroundTheme string
}

type CardViewData struct {
	App       models.App
	UserRole  string
	IsURLIcon bool
	IconName  string
}

type AppsViewData struct {
	Apps     []CardViewData
	UserRole string
}

type PageViewData struct {
	Session  auth.Session
	Settings DashboardSettings
}

func loadDashboardSettings() DashboardSettings {
	values, err := database.LoadSettings()
	if err != nil {
		return DashboardSettings{
			AppName:         "AMUD Dashboard",
			Tagline:         "High-Performance Intelligent Home Lab Cockpit",
			SearchEnabled:   true,
			ShowGreeting:    true,
			ShowClock:       true,
			BackgroundTheme: "aurora",
		}
	}

	searchEnabled, _ := strconv.ParseBool(values["search_enabled"])
	showGreeting, _ := strconv.ParseBool(values["show_greeting"])
	showClock, _ := strconv.ParseBool(values["show_clock"])

	return DashboardSettings{
		AppName:         values["app_name"],
		Tagline:         values["tagline"],
		SearchEnabled:   searchEnabled,
		ShowGreeting:    showGreeting,
		ShowClock:       showClock,
		BackgroundTheme: values["background_theme"],
	}
}

func HandleIndex(w http.ResponseWriter, r *http.Request) {
	if r.URL.Path != "/" {
		http.NotFound(w, r)
		return
	}

	session, _ := auth.GetSession(r)
	settings := loadDashboardSettings()

	w.Header().Set("Content-Type", "text/html; charset=utf-8")
	err := Templates.ExecuteTemplate(w, "base.html", PageViewData{Session: session, Settings: settings})
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
	}
}

func HandleGetApps(w http.ResponseWriter, r *http.Request) {
	category := r.URL.Query().Get("category")
	query := r.URL.Query().Get("q")

	apps, err := fetchAppsFiltered(category, query)
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}

	session, _ := auth.GetSession(r)

	// Populate Online statuses and wrap inside CardViewData
	var cardList []CardViewData
	for _, a := range apps {
		if val, ok := services.AppStatuses.Load(a.ID); ok {
			a.Online = val.(bool)
		} else {
			a.Online = false
		}

		isURL := strings.HasPrefix(a.Icon, "http://") || strings.HasPrefix(a.Icon, "https://") || strings.HasPrefix(a.Icon, "/")
		iconName := a.Icon
		if iconName == "" {
			iconName = "globe"
		}

		cardList = append(cardList, CardViewData{
			App:       a,
			UserRole:  session.Role,
			IsURLIcon: isURL,
			IconName:  iconName,
		})
	}

	w.Header().Set("Content-Type", "text/html; charset=utf-8")
	err = Templates.ExecuteTemplate(w, "dashboard.html", AppsViewData{
		Apps:     cardList,
		UserRole: session.Role,
	})
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
	}
}

func HandleCategories(w http.ResponseWriter, r *http.Request) {
	rows, err := database.DB.Query("SELECT DISTINCT category FROM apps WHERE category IS NOT NULL AND category != ''")
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	defer rows.Close()

	var categories []string
	for rows.Next() {
		var cat string
		if err := rows.Scan(&cat); err == nil {
			categories = append(categories, cat)
		}
	}

	w.Header().Set("Content-Type", "text/html; charset=utf-8")
	err = Templates.ExecuteTemplate(w, "categories.html", categories)
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
	}
}

func HandleTelemetry(w http.ResponseWriter, r *http.Request) {
	stats := services.GetSystemStats()
	w.Header().Set("Content-Type", "text/html; charset=utf-8")
	err := Templates.ExecuteTemplate(w, "telemetry.html", stats)
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
	}
}

func HandlePostApps(w http.ResponseWriter, r *http.Request) {
	// Restrict to Admin
	session, _ := auth.GetSession(r)
	if session.Role != "Admin" {
		http.Error(w, "unauthorized action", http.StatusForbidden)
		return
	}

	if err := r.ParseForm(); err != nil {
		http.Error(w, "failed to parse form data", http.StatusBadRequest)
		return
	}

	name := r.FormValue("name")
	url := r.FormValue("url")
	icon := r.FormValue("icon")
	category := r.FormValue("category")
	nodeTag := r.FormValue("node_tag")
	description := r.FormValue("description")

	if name == "" || url == "" {
		http.Error(w, "name and url are required fields", http.StatusBadRequest)
		return
	}
	if category == "" {
		category = "General"
	}
	if nodeTag == "" {
		nodeTag = "Local"
	}

	_, err := database.DB.Exec(
		"INSERT INTO apps (name, url, icon, description, category, node_tag) VALUES (?, ?, ?, ?, ?, ?)",
		name, url, icon, description, category, nodeTag,
	)
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}

	// Return updated apps HTMX fragment
	HandleGetApps(w, r)
}

func HandleRestartApp(w http.ResponseWriter, r *http.Request) {
	// Restrict to Admin
	session, _ := auth.GetSession(r)
	if session.Role != "Admin" {
		http.Error(w, "unauthorized action", http.StatusForbidden)
		return
	}

	idStr := r.PathValue("id")
	id, err := strconv.ParseInt(idStr, 10, 64)
	if err != nil {
		http.Error(w, "invalid application ID", http.StatusBadRequest)
		return
	}

	var name, url, icon, desc, cat, tag string
	err = database.DB.QueryRow("SELECT name, url, icon, description, category, node_tag FROM apps WHERE id = ?", id).Scan(&name, &url, &icon, &desc, &cat, &tag)
	if err != nil {
		http.Error(w, "application not found", http.StatusNotFound)
		return
	}

	go func(appName string) {
		client := &http.Client{
			Timeout: 10 * time.Second,
			Transport: &http.Transport{
				DialContext: func(ctx context.Context, network, addr string) (net.Conn, error) {
					return net.Dial("unix", "/var/run/docker.sock")
				},
			},
		}

		dockerName := strings.ToLower(strings.ReplaceAll(appName, " ", "-"))
		apiURL := fmt.Sprintf("http://localhost/containers/%s/restart", dockerName)

		resp, err := client.Post(apiURL, "application/json", nil)
		if err == nil {
			resp.Body.Close()
		}
	}(name)

	app := models.App{
		ID:          id,
		Name:        name,
		URL:         url,
		Icon:        icon,
		Description: desc,
		Category:    cat,
		NodeTag:     tag,
		Online:      true,
	}

	isURL := strings.HasPrefix(icon, "http://") || strings.HasPrefix(icon, "https://") || strings.HasPrefix(icon, "/")
	iconName := icon
	if iconName == "" {
		iconName = "globe"
	}

	w.Header().Set("Content-Type", "text/html; charset=utf-8")
	err = Templates.ExecuteTemplate(w, "card", CardViewData{
		App:       app,
		UserRole:  session.Role,
		IsURLIcon: isURL,
		IconName:  iconName,
	})
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
	}
}

func HandleLogin(w http.ResponseWriter, r *http.Request) {
	if r.Method == http.MethodGet {
		w.Header().Set("Content-Type", "text/html; charset=utf-8")
		_ = Templates.ExecuteTemplate(w, "login.html", nil)
		return
	}

	_ = r.ParseForm()
	username := r.FormValue("username")
	password := r.FormValue("password")

	var role, pwhash string
	err := database.DB.QueryRow("SELECT role, password_hash FROM users WHERE username = ?", username).Scan(&role, &pwhash)
	if err != nil || pwhash != auth.HashSha256(password) {
		w.Header().Set("Content-Type", "text/html; charset=utf-8")
		_ = Templates.ExecuteTemplate(w, "login.html", "Invalid username or password.")
		return
	}

	// Create session
	token := auth.CreateSession(username, role)
	http.SetCookie(w, &http.Cookie{
		Name:     "amud_session",
		Value:    token,
		Path:     "/",
		Expires:  time.Now().Add(24 * time.Hour),
		HttpOnly: true,
	})

	http.Redirect(w, r, "/", http.StatusSeeOther)
}

func HandleLogout(w http.ResponseWriter, r *http.Request) {
	cookie, err := r.Cookie("amud_session")
	if err == nil {
		auth.RemoveSession(cookie.Value)
	}

	// Clear cookie
	http.SetCookie(w, &http.Cookie{
		Name:     "amud_session",
		Value:    "",
		Path:     "/",
		Expires:  time.Unix(0, 0),
		MaxAge:   -1,
		HttpOnly: true,
	})

	http.Redirect(w, r, "/", http.StatusSeeOther)
}

func fetchAppsFiltered(category, query string) ([]models.App, error) {
	var rows *sql.Rows
	var err error
	var args []any
	var conditions []string
	baseQuery := "SELECT id, name, url, icon, description, category, node_tag FROM apps"

	if category != "" {
		conditions = append(conditions, "category = ?")
		args = append(args, category)
	}
	if query != "" {
		conditions = append(conditions, "(LOWER(name) LIKE ? OR LOWER(url) LIKE ? OR LOWER(COALESCE(description, '')) LIKE ? OR LOWER(COALESCE(category, '')) LIKE ? OR LOWER(COALESCE(node_tag, '')) LIKE ?)")
		search := "%" + strings.ToLower(query) + "%"
		args = append(args, search, search, search, search, search)
	}

	if len(conditions) > 0 {
		baseQuery += " WHERE " + strings.Join(conditions, " AND ")
	}
	baseQuery += " ORDER BY id DESC"

	rows, err = database.DB.Query(baseQuery, args...)

	if err != nil {
		return nil, err
	}
	defer rows.Close()

	var apps []models.App
	for rows.Next() {
		var app models.App
		var icon, desc, cat, tag sql.NullString
		err := rows.Scan(&app.ID, &app.Name, &app.URL, &icon, &desc, &cat, &tag)
		if err != nil {
			return nil, err
		}
		if icon.Valid {
			app.Icon = icon.String
		}
		if desc.Valid {
			app.Description = desc.String
		}
		if cat.Valid {
			app.Category = cat.String
		}
		if tag.Valid {
			app.NodeTag = tag.String
		}
		apps = append(apps, app)
	}
	return apps, nil
}
