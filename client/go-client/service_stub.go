//go:build !windows

package main

import "fmt"

func handleServiceCommand(options RunOptions) error {
	return fmt.Errorf("windows service commands are only supported on windows")
}
