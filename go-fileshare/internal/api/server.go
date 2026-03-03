package api

import (
	"context"
	"fmt"
	"log"
	"net/http"
	"time"

	"github.com/gin-gonic/gin"
	"github.com/gorilla/websocket"
	"github.com/igrisv3/fileshare/internal/config"
	"github.com/igrisv3/fileshare/internal/discovery"
	"github.com/igrisv3/fileshare/internal/transfer"
)

type Server struct {
	cfg         *config.Config
	transferMgr *transfer.Manager
	discovery   *discovery.Service
	router      *gin.Engine
	server      *http.Server
	upgrader    websocket.Upgrader
}

func NewServer(cfg *config.Config, transferMgr *transfer.Manager, discoveryService *discovery.Service) *Server {
	gin.SetMode(gin.ReleaseMode)
	router := gin.Default()

	// Enable CORS
	router.Use(func(c *gin.Context) {
		c.Writer.Header().Set("Access-Control-Allow-Origin", "*")
		c.Writer.Header().Set("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS")
		c.Writer.Header().Set("Access-Control-Allow-Headers", "Content-Type, Authorization")
		if c.Request.Method == "OPTIONS" {
			c.AbortWithStatus(204)
			return
		}
		c.Next()
	})

	s := &Server{
		cfg:         cfg,
		transferMgr: transferMgr,
		discovery:   discoveryService,
		router:      router,
		upgrader: websocket.Upgrader{
			CheckOrigin: func(r *http.Request) bool { return true },
		},
	}

	s.setupRoutes()
	return s
}

func (s *Server) setupRoutes() {
	api := s.router.Group("/api")
	{
		// LocalSend protocol v2.1 endpoints
		localsend := api.Group("/localsend/v2")
		{
			localsend.GET("/info", s.handleInfo)
			localsend.POST("/register", s.handleRegister)
			localsend.POST("/prepare-upload", s.handlePrepareUpload)
			localsend.POST("/upload", s.handleUpload)
			localsend.POST("/cancel", s.handleCancel)
		}

		// Custom IGRIS endpoints
		igris := api.Group("/igris")
		{
			igris.GET("/devices", s.handleGetDevices)
			igris.GET("/transfers", s.handleGetTransfers)
			igris.GET("/transfer/:id", s.handleGetTransfer)
			igris.POST("/send", s.handleSendFiles)
			igris.DELETE("/transfer/:id", s.handleCancelTransfer)
			igris.GET("/ws", s.handleWebSocket)
		}
	}

	// Health check
	s.router.GET("/health", func(c *gin.Context) {
		c.JSON(200, gin.H{"status": "ok"})
	})
}

// LocalSend Protocol Handlers

func (s *Server) handleInfo(c *gin.Context) {
	c.JSON(200, gin.H{
		"alias":       s.cfg.DeviceName,
		"version":     "2.1",
		"deviceType":  "desktop",
		"fingerprint": s.discovery.GetFingerprint(),
		"port":        s.cfg.Port,
		"protocol":    "http",
		"download":    false,
	})
}

func (s *Server) handleRegister(c *gin.Context) {
	var req struct {
		Alias       string `json:"alias"`
		Version     string `json:"version"`
		DeviceType  string `json:"device_type"`
		Fingerprint string `json:"fingerprint"`
		Port        int    `json:"port"`
		Protocol    string `json:"protocol"`
	}

	if err := c.BindJSON(&req); err != nil {
		c.JSON(400, gin.H{"error": "invalid request"})
		return
	}

	log.Printf("[API] Device registered: %s (%s)", req.Alias, req.DeviceType)
	
	c.JSON(200, gin.H{
		"alias":       s.cfg.DeviceName,
		"version":     "2.1",
		"deviceType":  "desktop",
		"fingerprint": s.discovery.GetFingerprint(),
	})
}

func (s *Server) handlePrepareUpload(c *gin.Context) {
	var req struct {
		Info  map[string]interface{} `json:"info"`
		Files map[string]interface{} `json:"files"`
	}

	if err := c.BindJSON(&req); err != nil {
		c.JSON(400, gin.H{"error": "invalid request"})
		return
	}

	// Convert files to FileInfo
	files := make([]transfer.FileInfo, 0)
	for fileID, fileData := range req.Files {
		fileMap := fileData.(map[string]interface{})
		files = append(files, transfer.FileInfo{
			ID:       fileID,
			FileName: fileMap["fileName"].(string),
			Size:     int64(fileMap["size"].(float64)),
			FileType: fileMap["fileType"].(string),
		})
	}

	// Create transfer session
	fromDevice := req.Info["alias"].(string)
	session, err := s.transferMgr.CreateSession(fromDevice, files)
	if err != nil {
		c.JSON(500, gin.H{"error": err.Error()})
		return
	}

	// Build response with tokens
	fileTokens := make(map[string]string)
	for _, file := range session.Files {
		fileTokens[file.ID] = file.Token
	}

	log.Printf("[API] Upload prepared: session=%s, files=%d", session.SessionID, len(files))

	c.JSON(200, gin.H{
		"sessionId": session.SessionID,
		"files":     fileTokens,
	})
}

func (s *Server) handleUpload(c *gin.Context) {
	sessionID := c.Query("sessionId")
	fileID := c.Query("fileId")
	token := c.Query("token")

	if sessionID == "" || fileID == "" || token == "" {
		c.JSON(400, gin.H{"error": "missing parameters"})
		return
	}

	// Verify session and token
	session, ok := s.transferMgr.GetTransfer(sessionID)
	if !ok {
		c.JSON(404, gin.H{"error": "session not found"})
		return
	}

	// Find file and verify token
	var fileInfo *transfer.FileInfo
	for i := range session.Files {
		if session.Files[i].ID == fileID && session.Files[i].Token == token {
			fileInfo = &session.Files[i]
			break
		}
	}
	if fileInfo == nil {
		c.JSON(403, gin.H{"error": "invalid token"})
		return
	}

	// Update status
	s.transferMgr.SetStatus(sessionID, transfer.StatusInProgress, "")

	// Save file
	if err := s.transferMgr.SaveFile(sessionID, fileID, c.Request.Body, fileInfo.FileName); err != nil {
		log.Printf("[API] Upload failed: %v", err)
		s.transferMgr.SetStatus(sessionID, transfer.StatusFailed, err.Error())
		c.JSON(500, gin.H{"error": err.Error()})
		return
	}

	log.Printf("[API] File uploaded: %s (%.2f MB)", fileInfo.FileName, float64(fileInfo.Size)/1024/1024)

	c.JSON(200, gin.H{"status": "ok"})
}

func (s *Server) handleCancel(c *gin.Context) {
	sessionID := c.Query("sessionId")
	if sessionID == "" {
		c.JSON(400, gin.H{"error": "missing sessionId"})
		return
	}

	if err := s.transferMgr.CancelTransfer(sessionID); err != nil {
		c.JSON(404, gin.H{"error": err.Error()})
		return
	}

	log.Printf("[API] Transfer cancelled: %s", sessionID)
	c.JSON(200, gin.H{"status": "cancelled"})
}

// IGRIS Custom Handlers

func (s *Server) handleGetDevices(c *gin.Context) {
	devices := s.discovery.GetDevices()
	c.JSON(200, gin.H{"devices": devices})
}

func (s *Server) handleGetTransfers(c *gin.Context) {
	transfers := s.transferMgr.GetAllTransfers()
	c.JSON(200, gin.H{"transfers": transfers})
}

func (s *Server) handleGetTransfer(c *gin.Context) {
	id := c.Param("id")
	transfer, ok := s.transferMgr.GetTransfer(id)
	if !ok {
		c.JSON(404, gin.H{"error": "transfer not found"})
		return
	}
	c.JSON(200, transfer)
}

func (s *Server) handleSendFiles(c *gin.Context) {
	var req struct {
		DeviceID  string   `json:"device_id"`
		FilePaths []string `json:"file_paths"`
	}

	if err := c.BindJSON(&req); err != nil {
		c.JSON(400, gin.H{"error": "invalid request"})
		return
	}

	// TODO: Implement sending files to remote device
	c.JSON(501, gin.H{"error": "not implemented yet"})
}

func (s *Server) handleCancelTransfer(c *gin.Context) {
	id := c.Param("id")
	if err := s.transferMgr.CancelTransfer(id); err != nil {
		c.JSON(404, gin.H{"error": err.Error()})
		return
	}
	c.JSON(200, gin.H{"status": "cancelled"})
}

func (s *Server) handleWebSocket(c *gin.Context) {
	conn, err := s.upgrader.Upgrade(c.Writer, c.Request, nil)
	if err != nil {
		log.Printf("[WS] Upgrade failed: %v", err)
		return
	}
	defer conn.Close()

	log.Println("[WS] Client connected")

	// Send updates every second
	ticker := time.NewTicker(1 * time.Second)
	defer ticker.Stop()

	for {
		select {
		case <-ticker.C:
			transfers := s.transferMgr.GetAllTransfers()
			if err := conn.WriteJSON(gin.H{"transfers": transfers}); err != nil {
				return
			}
		}
	}
}

func (s *Server) Start() error {
	addr := fmt.Sprintf(":%d", s.cfg.Port)
	s.server = &http.Server{
		Addr:    addr,
		Handler: s.router,
	}

	log.Printf("[API] Server starting on %s", addr)
	
	go func() {
		if err := s.server.ListenAndServe(); err != nil && err != http.ErrServerClosed {
			log.Fatalf("[API] Server error: %v", err)
		}
	}()

	return nil
}

func (s *Server) Stop() error {
	if s.server == nil {
		return nil
	}

	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	log.Println("[API] Server stopping...")
	return s.server.Shutdown(ctx)
}
