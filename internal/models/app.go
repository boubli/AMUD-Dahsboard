package models

type App struct {
	ID          int64  `json:"id"`
	Name        string `json:"name"`
	URL         string `json:"url"`
	Icon        string `json:"icon"`
	Description string `json:"description"`
	Category    string `json:"category"`
	NodeTag     string `json:"node_tag"`
	Online      bool   `json:"online"`
}
