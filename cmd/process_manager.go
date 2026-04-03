package main

import (
	"bufio"
	"context"
	"fmt"
	"io"
	"os"
	"os/exec"
	"runtime"
	"strings"
	"sync"
	"syscall"
	"time"
)

// ManagedProcess represents a running command and a channel that will be
// closed when the process exits. The exit error (if any) will be sent on the
// done channel.
type ManagedProcess struct {
	Cmd   *exec.Cmd
	Label string
	done  chan error
}

// ProcessManager manages multiple spawned processes: starting, tracking and
// shutting them down.
type ProcessManager struct {
	mu    sync.Mutex
	procs []*ManagedProcess
}

// NewProcessManager constructs an empty ProcessManager.
func NewProcessManager() *ProcessManager {
	return &ProcessManager{
		procs: make([]*ManagedProcess, 0),
	}
}

// SpawnCmds starts the provided shell commands. It kills any previously
// running processes managed by this manager before starting new ones.
// The function returns immediately after starting processes.
//
// Notes:
//   - Commands are executed via the platform shell ("sh -c" on unix, "cmd /C" on windows).
//   - Stdout and stderr from each command are streamed to the manager's stdout
//     with a simple label header so outputs from different commands can be
//     distinguished.
//   - Use WaitAny or KillAll to synchronise or stop processes.
func (pm *ProcessManager) SpawnCmds(ctx context.Context, cmds []string) error {
	// Ensure old processes are stopped first.
	if err := pm.KillAll(); err != nil {
		// Continue even if kill reports an error; we still want to spawn new cmds.
		fmt.Fprintf(os.Stderr, "warning: KillAll returned error: %v\n", err)
	}

	pm.mu.Lock()
	defer pm.mu.Unlock()

	for _, c := range cmds {
		c = strings.TrimSpace(c)
		if c == "" {
			continue
		}
		mp, err := startProcess(ctx, c)
		if err != nil {
			// If a particular process fails to start, attempt to kill any
			// processes we've already started for this batch, then return error.
			go func() {
				// best-effort cleanup without holding lock (KillAll will lock)
				_ = pm.KillAll()
			}()
			return fmt.Errorf("failed to start command %q: %w", c, err)
		}
		pm.procs = append(pm.procs, mp)
	}

	return nil
}

// WaitAny waits until any managed process exits and returns the process label
// and its exit error (nil if exit code 0). If no processes are running it
// returns immediately with empty label and nil error.
func (pm *ProcessManager) WaitAny() (label string, exitErr error) {
	pm.mu.Lock()
	procs := append([]*ManagedProcess(nil), pm.procs...)
	pm.mu.Unlock()

	if len(procs) == 0 {
		return "", nil
	}

	// Build a fan-in channel to receive the first exit.
	ch := make(chan struct {
		label string
		err   error
	}, 1)

	var once sync.Once

	for _, p := range procs {
		go func(mp *ManagedProcess) {
			err := <-mp.done
			once.Do(func() {
				ch <- struct {
					label string
					err   error
				}{mp.Label, err}
			})
		}(p)
	}

	res := <-ch
	return res.label, res.err
}

// KillAll attempts to gracefully terminate all managed processes, falling
// back to force kill when needed. It waits briefly for processes to exit.
//
// Returns an aggregated error if any kills failed.
func (pm *ProcessManager) KillAll() error {
	pm.mu.Lock()
	procs := pm.procs
	pm.procs = nil // detach slice immediately
	pm.mu.Unlock()

	if len(procs) == 0 {
		return nil
	}

	var wg sync.WaitGroup
	errs := make(chan error, len(procs))

	// Attempt graceful shutdown
	for _, p := range procs {
		wg.Add(1)
		go func(mp *ManagedProcess) {
			defer wg.Done()
			if mp.Cmd == nil || mp.Cmd.Process == nil {
				return
			}

			// First try to send SIGTERM (unix) or kill (windows)
			if runtime.GOOS == "windows" {
				// Windows doesn't have SIGTERM; Kill is the usual approach.
				if err := mp.Cmd.Process.Kill(); err != nil {
					errs <- fmt.Errorf("kill %s: %w", mp.Label, err)
					return
				}
			} else {
				// Try polite shutdown
				if err := mp.Cmd.Process.Signal(syscall.SIGTERM); err != nil {
					// If signalling fails, try Kill
					if err2 := mp.Cmd.Process.Kill(); err2 != nil {
						errs <- fmt.Errorf("term+kill %s: %v, %v", mp.Label, err, err2)
						return
					}
				}
			}

			// Wait for process to exit (with timeout)
			select {
			case <-mp.done:
				return
			case <-time.After(1500 * time.Millisecond):
				// If process still hasn't exited, force kill.
				if mp.Cmd.Process != nil {
					if err := mp.Cmd.Process.Kill(); err != nil {
						errs <- fmt.Errorf("force-kill %s: %w", mp.Label, err)
						return
					}
				}
				// Wait a bit more for the done channel to close
				select {
				case <-mp.done:
					return
				case <-time.After(500 * time.Millisecond):
					errs <- fmt.Errorf("process %s did not exit in time", mp.Label)
				}
			}
		}(p)
	}

	wg.Wait()
	close(errs)

	var agg []string
	for e := range errs {
		agg = append(agg, e.Error())
	}

	if len(agg) > 0 {
		return fmt.Errorf("errors while killing processes: %s", strings.Join(agg, "; "))
	}
	return nil
}

// startProcess creates and starts a single command using the platform shell,
// and returns a ManagedProcess whose done channel will receive the process
// exit result.
func startProcess(ctx context.Context, command string) (*ManagedProcess, error) {
	var shell string
	var shellArg string

	if runtime.GOOS == "windows" {
		shell = "cmd"
		shellArg = "/C"
	} else {
		shell = "sh"
		shellArg = "-c"
	}

	// Prepare the command to run with the shell
	cmd := exec.CommandContext(ctx, shell, shellArg, command)
	// Create pipes for stdout and stderr
	stdout, err := cmd.StdoutPipe()
	if err != nil {
		return nil, fmt.Errorf("stdout pipe: %w", err)
	}
	stderr, err := cmd.StderrPipe()
	if err != nil {
		return nil, fmt.Errorf("stderr pipe: %w", err)
	}

	// Start the process
	if err := cmd.Start(); err != nil {
		return nil, fmt.Errorf("start: %w", err)
	}

	mp := &ManagedProcess{
		Cmd:   cmd,
		Label: command,
		done:  make(chan error, 1),
	}

	// Stream stdout and stderr
	go streamOutput(mp.Label, "stdout", stdout, os.Stdout)
	go streamOutput(mp.Label, "stderr", stderr, os.Stdout)

	// Wait goroutine
	go func() {
		err := cmd.Wait()
		// send the error (nil on success)
		mp.done <- err
		close(mp.done)
	}()

	return mp, nil
}

// streamOutput reads lines from r and writes them to dst with a label prefix.
// It's resilient to io errors and will return when the reader is closed.
func streamOutput(label, streamName string, r io.Reader, dst io.Writer) {
	scanner := bufio.NewScanner(r)
	prefix := fmt.Sprintf("[%s][%s]: ", label, streamName)
	firstPrinted := false

	for scanner.Scan() {
		line := scanner.Text()
		if !firstPrinted {
			// Print a header for the command once
			fmt.Fprintln(dst, fmt.Sprintf("%s", label))
			firstPrinted = true
		}
		fmt.Fprintln(dst, prefix+line)
	}

	// If scanner encountered an error, show it
	if err := scanner.Err(); err != nil && err != io.EOF {
		fmt.Fprintln(dst, prefix+"(error reading output):", err)
	}
}
