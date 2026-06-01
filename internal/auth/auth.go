package auth

import (
	"crypto/sha256"
	"encoding/hex"
	"net/http"
	"sync"
	"time"
)

type Session struct {
	Username string
	Role     string
	Expires  time.Time
}

var (
	sessions     = make(map[string]Session)
	sessionMutex sync.RWMutex
)

func CreateSession(username string, role string) string {
	sessionMutex.Lock()
	defer sessionMutex.Unlock()

	h := sha256.New()
	h.Write([]byte(username + time.Now().String()))
	token := hex.EncodeToString(h.Sum(nil))

	sessions[token] = Session{
		Username: username,
		Role:     role,
		Expires:  time.Now().Add(24 * time.Hour),
	}
	return token
}

func GetSession(r *http.Request) (Session, bool) {
	cookie, err := r.Cookie("amud_session")
	if err != nil {
		// No session means unauthenticated. We treat them as "Guest"
		return Session{Role: "Guest"}, false
	}

	sessionMutex.RLock()
	defer sessionMutex.RUnlock()

	session, exists := sessions[cookie.Value]
	if !exists || time.Now().After(session.Expires) {
		return Session{Role: "Guest"}, false
	}
	return session, true
}

func RemoveSession(token string) {
	sessionMutex.Lock()
	defer sessionMutex.Unlock()
	delete(sessions, token)
}

func HashSha256(data string) string {
	h := sha256.New()
	h.Write([]byte(data))
	return hex.EncodeToString(h.Sum(nil))
}
