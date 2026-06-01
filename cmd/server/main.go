package main

import (
	"flag"
	"html/template"
	"log/slog"
	"net/http"
	"os"

	"github.com/boubli/AMUD-Dashboard/internal/database"
	"github.com/boubli/AMUD-Dashboard/internal/discovery"
	"github.com/boubli/AMUD-Dashboard/internal/handlers"
	"github.com/boubli/AMUD-Dashboard/internal/services"
	"github.com/boubli/AMUD-Dashboard/ui"
)

func main() {
	// Parse debugging flag
	debugFlag := flag.Bool("debug", false, "Enable verbose debug-level logging")
	flag.Parse()

	// Initialize structured logging level
	logLevel := slog.LevelInfo
	if *debugFlag {
		logLevel = slog.LevelDebug
	}

	logger := slog.New(slog.NewTextHandler(os.Stderr, &slog.HandlerOptions{
		Level: logLevel,
	}))
	slog.SetDefault(logger)

	slog.Info("AMUD Server starting up...")

	// Initialize database
	database.InitDB()

	// Start background ping engine
	services.StartPingEngine()

	// Start container auto-discovery
	discovery.StartAutoDiscovery()

	// Parse templates from embedded FS
	var err error
	handlers.Templates, err = template.ParseFS(ui.TemplateFS, 
		"templates/base.html", 
		"templates/dashboard.html", 
		"templates/telemetry.html",
		"templates/login.html",
		"templates/categories.html",
	)
	if err != nil {
		slog.Error("Failed to parse embedded HTML templates", "error", err)
		os.Exit(1)
	}

	// Register routes using Go 1.22 routing
	mux := http.NewServeMux()
	mux.HandleFunc("GET /", handlers.HandleIndex)
	mux.HandleFunc("GET /apps", handlers.HandleGetApps)
	mux.HandleFunc("GET /apps/status", handlers.HandleGetApps)
	mux.HandleFunc("POST /apps", handlers.HandlePostApps)
	mux.HandleFunc("POST /apps/{id}/restart", handlers.HandleRestartApp)
	mux.HandleFunc("GET /telemetry", handlers.HandleTelemetry)
	mux.HandleFunc("GET /categories", handlers.HandleCategories)
	
	// Authentication routes
	mux.HandleFunc("GET /login", handlers.HandleLogin)
	mux.HandleFunc("POST /login", handlers.HandleLogin)
	mux.HandleFunc("GET /logout", handlers.HandleLogout)

	// Admin database backup and restore endpoints
	mux.HandleFunc("GET /admin/backup", handlers.HandleBackup)
	mux.HandleFunc("POST /admin/restore", handlers.HandleRestore)

	// Serve embedded static files
	mux.Handle("GET /static/", http.FileServer(http.FS(ui.StaticFS)))

	// Wrap mux handler in logging and recovery middleware chain
	var handler http.Handler = mux
	handler = loggingMiddleware(handler)
	handler = handlers.RecoveryMiddleware(handler)

	// Start HTTP server listener
	port := os.Getenv("PORT")
	if port == "" {
		port = "8000"
	}

	slog.Info("HTTP server listener online", "port", port)
	if err := http.ListenAndServe(":"+port, handler); err != nil {
		slog.Error("HTTP server crash failure", "error", err)
		os.Exit(1)
	}
}

// Request logging middleware. Writes request metadata at DEBUG level.
func loggingMiddleware(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		slog.Debug("HTTP request intercepted", 
			"method", r.Method, 
			"path", r.URL.Path, 
			"remote_address", r.RemoteAddr,
		)
		next.ServeHTTP(w, r)
	})
}
