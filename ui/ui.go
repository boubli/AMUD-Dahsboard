package ui

import "embed"

//go:embed templates/*.html templates/components/*.html
var TemplateFS embed.FS

//go:embed static/*
var StaticFS embed.FS
