package config

import (
	"encoding/json"
	"os"
	"path/filepath"
)

type Config struct {
	DeviceName       string `json:"device_name"`
	Port             int    `json:"port"`
	DownloadDir      string `json:"download_dir"`
	AutoAcceptTrusted bool  `json:"auto_accept_trusted"`
	MaxTransferSize  int64  `json:"max_transfer_size"`
	ChunkSize        int    `json:"chunk_size"`
	Enabled          bool   `json:"enabled"`
}

func Default() *Config {
	homeDir, _ := os.UserHomeDir()
	return &Config{
		DeviceName:       "IGRIS",
		Port:             53317,
		DownloadDir:      filepath.Join(homeDir, "Downloads", "IGRIS"),
		AutoAcceptTrusted: false,
		MaxTransferSize:  10 * 1024 * 1024 * 1024, // 10GB
		ChunkSize:        64 * 1024,                // 64KB
		Enabled:          true,
	}
}

func Load(path string) (*Config, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return nil, err
	}

	cfg := Default()
	if err := json.Unmarshal(data, cfg); err != nil {
		return nil, err
	}

	// Ensure download directory exists
	if err := os.MkdirAll(cfg.DownloadDir, 0755); err != nil {
		return nil, err
	}

	return cfg, nil
}

func (c *Config) Save(path string) error {
	data, err := json.MarshalIndent(c, "", "  ")
	if err != nil {
		return err
	}
	return os.WriteFile(path, data, 0644)
}
