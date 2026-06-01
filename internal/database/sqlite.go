package database

import (
	"crypto/sha256"
	"database/sql"
	"encoding/hex"
	"log/slog"
	"os"
	"path/filepath"

	_ "modernc.org/sqlite"
)

var DB *sql.DB

func InitDB() {
	dbPath := os.Getenv("DB_PATH")
	if dbPath == "" {
		dbPath = "data/amud.db"
	}

	slog.Info("Initializing SQLite connection pool", "path", dbPath)

	// Ensure directory exists
	dir := filepath.Dir(dbPath)
	if err := os.MkdirAll(dir, 0755); err != nil {
		slog.Error("Failed to create database directory", "error", err)
		os.Exit(1)
	}

	var err error
	DB, err = sql.Open("sqlite", dbPath)
	if err != nil {
		slog.Error("Failed to open database file", "error", err)
		os.Exit(1)
	}

	// Configure SQLite for WAL journaling and speed
	_, err = DB.Exec("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
	if err != nil {
		slog.Error("Failed to configure SQLite WAL mode", "error", err)
		os.Exit(1)
	}

	slog.Debug("SQLite WAL mode and foreign keys configured successfully")

	// Create tables
	migrations := []string{
		`CREATE TABLE IF NOT EXISTS apps (
			id INTEGER PRIMARY KEY AUTOINCREMENT,
			name TEXT NOT NULL,
			url TEXT NOT NULL,
			icon TEXT,
			description TEXT
		);`,
		`CREATE TABLE IF NOT EXISTS users (
			id INTEGER PRIMARY KEY AUTOINCREMENT,
			username TEXT UNIQUE NOT NULL,
			password_hash TEXT NOT NULL,
			role TEXT NOT NULL DEFAULT 'Guest'
		);`,
	}

	for i, q := range migrations {
		if _, err := DB.Exec(q); err != nil {
			slog.Error("Failed to execute database migration schema", "index", i, "error", err)
			os.Exit(1)
		}
	}

	// Dynamic column migrations (ignore errors if columns exist)
	_, _ = DB.Exec("ALTER TABLE apps ADD COLUMN category TEXT DEFAULT 'General';")
	_, _ = DB.Exec("ALTER TABLE apps ADD COLUMN node_tag TEXT DEFAULT 'Local';")

	slog.Debug("SQLite structural schema migrations validated")

	// Seed default apps if empty
	var appCount int
	err = DB.QueryRow("SELECT COUNT(*) FROM apps").Scan(&appCount)
	if err == nil && appCount == 0 {
		slog.Info("Seeding initial application database records")
		seeds := []struct {
			name, url, icon, desc, cat, tag string
		}{
			{"Proxmox VE", "https://proxmox.local:8006", "server", "Virtualization management platform", "Infrastructure", "Proxmox"},
			{"Portainer", "http://portainer.local:9000", "container", "Docker container management dashboard", "Infrastructure", "Local"},
			{"Jellyfin", "http://jellyfin.local:8096", "tv", "Open-source media system", "Media", "Local"},
		}
		for _, s := range seeds {
			_, err = DB.Exec(
				"INSERT INTO apps (name, url, icon, description, category, node_tag) VALUES (?, ?, ?, ?, ?, ?)",
				s.name, s.url, s.icon, s.desc, s.cat, s.tag,
			)
			if err != nil {
				slog.Error("Failed to insert seed application", "name", s.name, "error", err)
			}
		}
	}

	// Seed default users if empty
	var userCount int
	err = DB.QueryRow("SELECT COUNT(*) FROM users").Scan(&userCount)
	if err == nil && userCount == 0 {
		slog.Info("Seeding initial security user roles")
		adminHash := hashSha256("admin")
		guestHash := hashSha256("guest")
		_, _ = DB.Exec("INSERT INTO users (username, password_hash, role) VALUES (?, ?, ?)", "admin", adminHash, "Admin")
		_, _ = DB.Exec("INSERT INTO users (username, password_hash, role) VALUES (?, ?, ?)", "guest", guestHash, "Guest")
	}
}

func hashSha256(data string) string {
	h := sha256.New()
	h.Write([]byte(data))
	return hex.EncodeToString(h.Sum(nil))
}
