package handlers

import (
	"log/slog"
	"net/http"
	"runtime/debug"
)

// RecoveryMiddleware intercepts unexpected panics, logs the stack trace structured, and returns an HTMX-safe response.
func RecoveryMiddleware(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		defer func() {
			if err := recover(); err != nil {
				// Capture full stack trace
				stack := debug.Stack()
				slog.Error("Recovered from server panic", 
					"error", err, 
					"path", r.URL.Path, 
					"method", r.Method, 
					"stack", string(stack),
				)

				// HTMX partial swap alert fallback
				if r.Header.Get("HX-Request") == "true" {
					w.Header().Set("Content-Type", "text/html; charset=utf-8")
					w.WriteHeader(http.StatusInternalServerError)
					_, _ = w.Write([]byte(`<div style="background: rgba(239, 68, 68, 0.1); border: 1px solid var(--danger); color: var(--danger); padding: 1rem; border-radius: 0.5rem; margin: 1rem 0; font-size: 0.9rem; width: 100%; text-align: center;">
						<strong>Internal System Error</strong><br>
						<span style="font-size: 0.8rem; opacity: 0.85;">An unexpected panic occurred during execution. Stack trace logged to console.</span>
					</div>`))
					return
				}

				// Non-HTMX standard response
				http.Error(w, "Internal Server Error", http.StatusInternalServerError)
			}
		}()
		next.ServeHTTP(w, r)
	})
}
