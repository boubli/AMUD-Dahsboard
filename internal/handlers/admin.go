package handlers

import (
	"io"
	"log/slog"
	"net/http"
	"os"
	"path/filepath"
	"strconv"

	"github.com/boubli/AMUD-Dashboard/internal/auth"
	"github.com/boubli/AMUD-Dashboard/internal/database"
)

// HandleBackup streams the SQLite database file as a downloadable file attachment.
func HandleBackup(w http.ResponseWriter, r *http.Request) {
	// Restrict to Admin
	session, _ := auth.GetSession(r)
	if session.Role != "Admin" {
		http.Error(w, "unauthorized action", http.StatusForbidden)
		return
	}

	dbPath := os.Getenv("DB_PATH")
	if dbPath == "" {
		dbPath = "data/amud.db"
	}

	file, err := os.Open(dbPath)
	if err != nil {
		slog.Error("backup failed: could not open db file", "error", err)
		http.Error(w, "failed to open database file", http.StatusInternalServerError)
		return
	}
	defer file.Close()

	w.Header().Set("Content-Disposition", "attachment; filename=amud.db")
	w.Header().Set("Content-Type", "application/octet-stream")

	_, err = io.Copy(w, file)
	if err != nil {
		slog.Error("backup failed: write stream error", "error", err)
	}
}

// HandleRestore accepts a multipart uploaded SQLite database, overrides the current one, and triggers reload.
func HandleRestore(w http.ResponseWriter, r *http.Request) {
	// Restrict to Admin
	session, _ := auth.GetSession(r)
	if session.Role != "Admin" {
		http.Error(w, "unauthorized action", http.StatusForbidden)
		return
	}

	// Limit upload size to 10MB to enforce ultra-lean constraints
	err := r.ParseMultipartForm(10 << 20)
	if err != nil {
		http.Error(w, "upload size exceeds limit", http.StatusBadRequest)
		return
	}

	uploadFile, _, err := r.FormFile("database")
	if err != nil {
		http.Error(w, "failed to read uploaded database file", http.StatusBadRequest)
		return
	}
	defer uploadFile.Close()

	dbPath := os.Getenv("DB_PATH")
	if dbPath == "" {
		dbPath = "data/amud.db"
	}

	// Ensure destination directory exists
	dir := filepath.Dir(dbPath)
	_ = os.MkdirAll(dir, 0755)

	// Close database connection before swapping file
	if database.DB != nil {
		_ = database.DB.Close()
	}

	// Create temporary db swap destination
	tempFile, err := os.Create(dbPath + ".tmp")
	if err != nil {
		slog.Error("restore failed: could not create temp file", "error", err)
		database.InitDB() // Re-open original connection
		http.Error(w, "internal disk write failure", http.StatusInternalServerError)
		return
	}

	_, err = io.Copy(tempFile, uploadFile)
	tempFile.Close()
	if err != nil {
		slog.Error("restore failed: write stream error", "error", err)
		os.Remove(dbPath + ".tmp")
		database.InitDB() // Re-open original connection
		http.Error(w, "internal disk copy failure", http.StatusInternalServerError)
		return
	}

	// Overwrite database file
	err = os.Rename(dbPath+".tmp", dbPath)
	if err != nil {
		slog.Error("restore failed: could not rename temp file", "error", err)
		os.Remove(dbPath + ".tmp")
		database.InitDB() // Re-open original connection
		http.Error(w, "failed to overwrite database file", http.StatusInternalServerError)
		return
	}

	slog.Info("database file swapped successfully, re-opening connection pool")
	
	// Reinitialize the SQLite database pool and migrate
	database.InitDB()

	// Trigger grid reload on frontend by setting HTMX response headers
	w.Header().Set("HX-Trigger", "reload-grid")
	w.Header().Set("Content-Type", "text/html; charset=utf-8")
	_, _ = w.Write([]byte(`<div style="background: rgba(16, 185, 129, 0.1); border: 1px solid var(--success); color: var(--success); padding: 0.75rem 1rem; border-radius: 0.5rem; font-size: 0.875rem; text-align: center; width: 100%; margin-top: 1rem;">
		Database restored successfully. Reloading...
	</div>`))
}

func HandleSettings(w http.ResponseWriter, r *http.Request) {
	session, _ := auth.GetSession(r)
	if session.Role != "Admin" {
		http.Error(w, "unauthorized action", http.StatusForbidden)
		return
	}

	if err := r.ParseForm(); err != nil {
		http.Error(w, "failed to parse settings form", http.StatusBadRequest)
		return
	}

	settings := map[string]string{
		"app_name":         r.FormValue("app_name"),
		"tagline":          r.FormValue("tagline"),
		"background_theme": r.FormValue("background_theme"),
		"search_enabled":   strconv.FormatBool(r.FormValue("search_enabled") == "on" || r.FormValue("search_enabled") == "true"),
		"show_greeting":    strconv.FormatBool(r.FormValue("show_greeting") == "on" || r.FormValue("show_greeting") == "true"),
		"show_clock":       strconv.FormatBool(r.FormValue("show_clock") == "on" || r.FormValue("show_clock") == "true"),
	}

	if settings["app_name"] == "" {
		settings["app_name"] = "AMUD Dashboard"
	}
	if settings["tagline"] == "" {
		settings["tagline"] = "High-Performance Intelligent Home Lab Cockpit"
	}
	if settings["background_theme"] == "" {
		settings["background_theme"] = "aurora"
	}

	if err := database.SaveSettings(settings); err != nil {
		slog.Error("failed to persist dashboard settings", "error", err)
		http.Error(w, "failed to save settings", http.StatusInternalServerError)
		return
	}

	w.Header().Set("HX-Trigger", "settings-saved")
	w.Header().Set("Content-Type", "text/html; charset=utf-8")
	_, _ = w.Write([]byte(`<div class="settings-save-note">Settings saved. Refreshing dashboard...</div>`))
}

func HandleServiceStat(w http.ResponseWriter, r *http.Request) {
	service := r.PathValue("service")
	label := "Active"
	if service == "plex" || service == "jellyfin" {
		label = "0 Watching"
	} else if service == "proxmox" {
		label = "Healthy"
	}

	w.Header().Set("Content-Type", "text/html; charset=utf-8")
	_, _ = w.Write([]byte(`<span class="stat-pill" id="stat-` + service + `" hx-get="/api/stats/` + service + `" hx-trigger="every 10s" hx-swap="outerHTML">` + label + `</span>`))
}
