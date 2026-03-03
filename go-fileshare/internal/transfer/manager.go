package transfer

import (
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"sync"
	"time"

	"github.com/google/uuid"
	"github.com/igrisv3/fileshare/internal/config"
)

type TransferStatus string

const (
	StatusPending    TransferStatus = "pending"
	StatusInProgress TransferStatus = "in_progress"
	StatusCompleted  TransferStatus = "completed"
	StatusFailed     TransferStatus = "failed"
	StatusCancelled  TransferStatus = "cancelled"
)

type FileInfo struct {
	ID       string `json:"id"`
	FileName string `json:"file_name"`
	Size     int64  `json:"size"`
	FileType string `json:"file_type"`
	SHA256   string `json:"sha256,omitempty"`
	Token    string `json:"token"`
}

type Transfer struct {
	SessionID    string         `json:"session_id"`
	Files        []FileInfo     `json:"files"`
	Status       TransferStatus `json:"status"`
	BytesSent    int64          `json:"bytes_sent"`
	TotalBytes   int64          `json:"total_bytes"`
	StartTime    time.Time      `json:"start_time"`
	EndTime      *time.Time     `json:"end_time,omitempty"`
	FromDevice   string         `json:"from_device"`
	ToDevice     string         `json:"to_device"`
	ErrorMessage string         `json:"error_message,omitempty"`
}

type Manager struct {
	cfg       *config.Config
	transfers map[string]*Transfer
	mutex     sync.RWMutex
}

func NewManager(cfg *config.Config) *Manager {
	return &Manager{
		cfg:       cfg,
		transfers: make(map[string]*Transfer),
	}
}

func (m *Manager) CreateSession(fromDevice string, files []FileInfo) (*Transfer, error) {
	sessionID := uuid.New().String()
	
	var totalBytes int64
	for _, file := range files {
		totalBytes += file.Size
		// Generate token for each file
		file.Token = generateToken()
	}

	transfer := &Transfer{
		SessionID:  sessionID,
		Files:      files,
		Status:     StatusPending,
		TotalBytes: totalBytes,
		StartTime:  time.Now(),
		FromDevice: fromDevice,
	}

	m.mutex.Lock()
	m.transfers[sessionID] = transfer
	m.mutex.Unlock()

	return transfer, nil
}

func (m *Manager) GetTransfer(sessionID string) (*Transfer, bool) {
	m.mutex.RLock()
	defer m.mutex.RUnlock()
	
	transfer, ok := m.transfers[sessionID]
	return transfer, ok
}

func (m *Manager) UpdateProgress(sessionID string, bytesTransferred int64) {
	m.mutex.Lock()
	defer m.mutex.Unlock()
	
	if transfer, ok := m.transfers[sessionID]; ok {
		transfer.BytesSent += bytesTransferred
		if transfer.BytesSent >= transfer.TotalBytes {
			transfer.Status = StatusCompleted
			now := time.Now()
			transfer.EndTime = &now
		}
	}
}

func (m *Manager) SetStatus(sessionID string, status TransferStatus, errorMsg string) {
	m.mutex.Lock()
	defer m.mutex.Unlock()
	
	if transfer, ok := m.transfers[sessionID]; ok {
		transfer.Status = status
		transfer.ErrorMessage = errorMsg
		if status == StatusCompleted || status == StatusFailed || status == StatusCancelled {
			now := time.Now()
			transfer.EndTime = &now
		}
	}
}

func (m *Manager) CancelTransfer(sessionID string) error {
	m.mutex.Lock()
	defer m.mutex.Unlock()
	
	transfer, ok := m.transfers[sessionID]
	if !ok {
		return fmt.Errorf("transfer not found")
	}
	
	transfer.Status = StatusCancelled
	now := time.Now()
	transfer.EndTime = &now
	
	return nil
}

func (m *Manager) SaveFile(sessionID, fileID string, reader io.Reader, fileName string) error {
	transfer, ok := m.GetTransfer(sessionID)
	if !ok {
		return fmt.Errorf("session not found")
	}

	// Find file info
	var fileInfo *FileInfo
	for i := range transfer.Files {
		if transfer.Files[i].ID == fileID {
			fileInfo = &transfer.Files[i]
			break
		}
	}
	if fileInfo == nil {
		return fmt.Errorf("file not found in session")
	}

	// Create destination path
	destPath := filepath.Join(m.cfg.DownloadDir, fileName)
	
	// Ensure directory exists
	if err := os.MkdirAll(filepath.Dir(destPath), 0755); err != nil {
		return err
	}

	// Create file
	file, err := os.Create(destPath)
	if err != nil {
		return err
	}
	defer file.Close()

	// Copy with progress tracking and checksum
	hash := sha256.New()
	multiWriter := io.MultiWriter(file, hash)
	
	written, err := io.Copy(multiWriter, reader)
	if err != nil {
		return err
	}

	// Verify size
	if written != fileInfo.Size {
		return fmt.Errorf("size mismatch: expected %d, got %d", fileInfo.Size, written)
	}

	// Verify checksum if provided
	if fileInfo.SHA256 != "" {
		actualHash := hex.EncodeToString(hash.Sum(nil))
		if actualHash != fileInfo.SHA256 {
			return fmt.Errorf("checksum mismatch")
		}
	}

	// Update progress
	m.UpdateProgress(sessionID, written)

	return nil
}

func (m *Manager) GetAllTransfers() []*Transfer {
	m.mutex.RLock()
	defer m.mutex.RUnlock()
	
	transfers := make([]*Transfer, 0, len(m.transfers))
	for _, t := range m.transfers {
		transfers = append(transfers, t)
	}
	return transfers
}

func generateToken() string {
	return uuid.New().String()
}

func (t *Transfer) Progress() float64 {
	if t.TotalBytes == 0 {
		return 0
	}
	return float64(t.BytesSent) / float64(t.TotalBytes) * 100
}
