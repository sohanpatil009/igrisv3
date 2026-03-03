package main

import (
	"flag"
	"log"
	"os"
	"os/signal"
	"syscall"

	"github.com/igrisv3/fileshare/internal/api"
	"github.com/igrisv3/fileshare/internal/config"
	"github.com/igrisv3/fileshare/internal/discovery"
	"github.com/igrisv3/fileshare/internal/transfer"
)

func main() {
	// Parse flags
	configPath := flag.String("config", "config.json", "Path to configuration file")
	port := flag.Int("port", 53317, "Port to listen on")
	deviceName := flag.String("name", "IGRIS", "Device name")
	flag.Parse()

	// Load configuration
	cfg, err := config.Load(*configPath)
	if err != nil {
		log.Printf("Failed to load config, using defaults: %v", err)
		cfg = config.Default()
	}
	cfg.Port = *port
	cfg.DeviceName = *deviceName

	// Initialize components
	log.Printf("Starting IGRIS File Share Backend v1.0.0")
	log.Printf("Device: %s, Port: %d", cfg.DeviceName, cfg.Port)

	// Create transfer manager
	transferMgr := transfer.NewManager(cfg)

	// Create mDNS discovery service
	discoveryService, err := discovery.NewService(cfg, transferMgr)
	if err != nil {
		log.Fatalf("Failed to create discovery service: %v", err)
	}

	// Start discovery
	if err := discoveryService.Start(); err != nil {
		log.Fatalf("Failed to start discovery: %v", err)
	}
	defer discoveryService.Stop()

	// Create and start HTTP API server
	apiServer := api.NewServer(cfg, transferMgr, discoveryService)
	go func() {
		if err := apiServer.Start(); err != nil {
			log.Fatalf("Failed to start API server: %v", err)
		}
	}()
	defer apiServer.Stop()

	log.Printf("✓ File share backend running on port %d", cfg.Port)
	log.Printf("✓ API available at http://localhost:%d", cfg.Port)
	log.Printf("✓ Download directory: %s", cfg.DownloadDir)
	log.Println("✓ Ready to discover devices on mobile hotspot")
	log.Println()
	log.Println("Press Ctrl+C to stop")

	// Wait for interrupt signal
	sigChan := make(chan os.Signal, 1)
	signal.Notify(sigChan, os.Interrupt, syscall.SIGTERM)
	<-sigChan

	log.Println("\nShutting down gracefully...")
}
