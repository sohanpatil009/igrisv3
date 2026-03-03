package discovery

import (
	"context"
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"log"
	"net"
	"sync"
	"time"

	"github.com/google/uuid"
	"github.com/grandcat/zeroconf"
	"github.com/igrisv3/fileshare/internal/config"
	"github.com/igrisv3/fileshare/internal/transfer"
)

type Device struct {
	ID          string    `json:"id"`
	Alias       string    `json:"alias"`
	IP          string    `json:"ip"`
	Port        int       `json:"port"`
	Fingerprint string    `json:"fingerprint"`
	DeviceType  string    `json:"device_type"`
	LastSeen    time.Time `json:"last_seen"`
	Protocol    string    `json:"protocol"`
}

type Service struct {
	cfg          *config.Config
	transferMgr  *transfer.Manager
	devices      map[string]*Device
	devicesMutex sync.RWMutex
	server       *zeroconf.Server
	fingerprint  string
	ctx          context.Context
	cancel       context.CancelFunc
}

func NewService(cfg *config.Config, transferMgr *transfer.Manager) (*Service, error) {
	ctx, cancel := context.WithCancel(context.Background())
	
	// Generate device fingerprint
	fingerprint := generateFingerprint(cfg.DeviceName)
	
	return &Service{
		cfg:         cfg,
		transferMgr: transferMgr,
		devices:     make(map[string]*Device),
		fingerprint: fingerprint,
		ctx:         ctx,
		cancel:      cancel,
	}, nil
}

func generateFingerprint(deviceName string) string {
	data := fmt.Sprintf("%s-%s-%d", deviceName, uuid.New().String(), time.Now().Unix())
	hash := sha256.Sum256([]byte(data))
	return hex.EncodeToString(hash[:])
}

func (s *Service) Start() error {
	// Start mDNS server for broadcasting
	if err := s.startBroadcasting(); err != nil {
		return fmt.Errorf("failed to start broadcasting: %w", err)
	}

	// Start mDNS browser for discovering peers
	go s.startDiscovery()

	log.Printf("[DISCOVERY] Started on port %d with fingerprint %s", s.cfg.Port, s.fingerprint[:16])
	return nil
}

func (s *Service) startBroadcasting() error {
	// Get local IP address
	ip, err := getLocalIP()
	if err != nil {
		return err
	}

	// Register mDNS service
	server, err := zeroconf.Register(
		s.cfg.DeviceName,           // Instance name
		"_localsend._tcp",          // Service type
		"local.",                   // Domain
		s.cfg.Port,                 // Port
		[]string{
			fmt.Sprintf("fingerprint=%s", s.fingerprint),
			fmt.Sprintf("protocol=http"),
			fmt.Sprintf("version=2.1"),
			fmt.Sprintf("deviceType=desktop"),
		},
		nil, // Use all network interfaces
	)
	if err != nil {
		return err
	}

	s.server = server
	log.Printf("[DISCOVERY] Broadcasting as '%s' on %s:%d", s.cfg.DeviceName, ip, s.cfg.Port)
	return nil
}

func (s *Service) startDiscovery() {
	resolver, err := zeroconf.NewResolver(nil)
	if err != nil {
		log.Printf("[DISCOVERY] Failed to create resolver: %v", err)
		return
	}

	entries := make(chan *zeroconf.ServiceEntry)
	
	go func() {
		for entry := range entries {
			s.handleDiscoveredDevice(entry)
		}
	}()

	// Browse for LocalSend services
	ctx := s.ctx
	if err := resolver.Browse(ctx, "_localsend._tcp", "local.", entries); err != nil {
		log.Printf("[DISCOVERY] Failed to browse: %v", err)
	}
}

func (s *Service) handleDiscoveredDevice(entry *zeroconf.ServiceEntry) {
	if len(entry.AddrIPv4) == 0 {
		return
	}

	ip := entry.AddrIPv4[0].String()
	deviceID := fmt.Sprintf("%s:%d", ip, entry.Port)

	// Skip self
	if entry.Instance == s.cfg.DeviceName {
		return
	}

	// Extract metadata
	fingerprint := ""
	protocol := "http"
	deviceType := "unknown"
	
	for _, txt := range entry.Text {
		if len(txt) > 12 && txt[:12] == "fingerprint=" {
			fingerprint = txt[12:]
		} else if len(txt) > 9 && txt[:9] == "protocol=" {
			protocol = txt[9:]
		} else if len(txt) > 11 && txt[:11] == "deviceType=" {
			deviceType = txt[11:]
		}
	}

	device := &Device{
		ID:          deviceID,
		Alias:       entry.Instance,
		IP:          ip,
		Port:        entry.Port,
		Fingerprint: fingerprint,
		DeviceType:  deviceType,
		Protocol:    protocol,
		LastSeen:    time.Now(),
	}

	s.devicesMutex.Lock()
	s.devices[deviceID] = device
	s.devicesMutex.Unlock()

	log.Printf("[DISCOVERY] Found device: %s (%s) at %s", device.Alias, device.DeviceType, deviceID)
}

func (s *Service) GetDevices() []*Device {
	s.devicesMutex.RLock()
	defer s.devicesMutex.RUnlock()

	devices := make([]*Device, 0, len(s.devices))
	now := time.Now()
	
	for _, device := range s.devices {
		// Only return devices seen in last 60 seconds
		if now.Sub(device.LastSeen) < 60*time.Second {
			devices = append(devices, device)
		}
	}
	
	return devices
}

func (s *Service) GetDevice(id string) (*Device, bool) {
	s.devicesMutex.RLock()
	defer s.devicesMutex.RUnlock()
	
	device, ok := s.devices[id]
	return device, ok
}

func (s *Service) Stop() {
	if s.server != nil {
		s.server.Shutdown()
	}
	s.cancel()
	log.Println("[DISCOVERY] Stopped")
}

func getLocalIP() (string, error) {
	addrs, err := net.InterfaceAddrs()
	if err != nil {
		return "", err
	}

	for _, addr := range addrs {
		if ipnet, ok := addr.(*net.IPNet); ok && !ipnet.IP.IsLoopback() {
			if ipnet.IP.To4() != nil {
				return ipnet.IP.String(), nil
			}
		}
	}
	
	return "", fmt.Errorf("no local IP found")
}

func (s *Service) GetFingerprint() string {
	return s.fingerprint
}
