package database

import "database/sql"

var defaultSettings = map[string]string{
	"app_name":         "AMUD Dashboard",
	"tagline":          "High-Performance Intelligent Home Lab Cockpit",
	"search_enabled":   "true",
	"show_greeting":    "true",
	"show_clock":       "true",
	"background_theme": "aurora",
}

func LoadSettings() (map[string]string, error) {
	rows, err := DB.Query("SELECT key, value FROM settings")
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	settings := make(map[string]string, len(defaultSettings))
	for key, value := range defaultSettings {
		settings[key] = value
	}

	for rows.Next() {
		var key string
		var value sql.NullString
		if err := rows.Scan(&key, &value); err != nil {
			return nil, err
		}
		if value.Valid {
			settings[key] = value.String
		}
	}

	return settings, nil
}

func SaveSettings(settings map[string]string) error {
	tx, err := DB.Begin()
	if err != nil {
		return err
	}
	defer tx.Rollback()

	for key, value := range settings {
		if _, err := tx.Exec(`
			INSERT INTO settings (key, value) VALUES (?, ?)
			ON CONFLICT(key) DO UPDATE SET value = excluded.value
		`, key, value); err != nil {
			return err
		}
	}

	return tx.Commit()
}
