package services

import (
	"bufio"
	"fmt"
	"math"
	"os"
	"strconv"
	"strings"
	"sync"
)

type SystemStats struct {
	CPU        int     // Percentage (0-100)
	RAM        int     // Percentage (0-100)
	RAMUsedGB  float64 // GB
	RAMTotalGB float64 // GB
}

var (
	lastTotalCpu uint64
	lastIdleCpu  uint64
	cpuMutex     sync.Mutex
)

func init() {
	// Initialize CPU readings
	total, idle, err := readProcStat()
	if err == nil {
		lastTotalCpu = total
		lastIdleCpu = idle
	}
}

func GetSystemStats() SystemStats {
	cpu := getCPUUsage()
	ramPercent, usedGB, totalGB := getRAMUsage()

	return SystemStats{
		CPU:        cpu,
		RAM:        ramPercent,
		RAMUsedGB:  usedGB,
		RAMTotalGB: totalGB,
	}
}

func getCPUUsage() int {
	cpuMutex.Lock()
	defer cpuMutex.Unlock()

	total, idle, err := readProcStat()
	if err != nil {
		// Fallback to mock on non-linux systems
		return 8
	}

	if lastTotalCpu == 0 {
		lastTotalCpu = total
		lastIdleCpu = idle
		return 8
	}

	deltaTotal := total - lastTotalCpu
	deltaIdle := idle - lastIdleCpu

	lastTotalCpu = total
	lastIdleCpu = idle

	if deltaTotal == 0 {
		return 0
	}

	usage := float64(deltaTotal-deltaIdle) / float64(deltaTotal) * 100
	if usage < 0 {
		usage = 0
	} else if usage > 100 {
		usage = 100
	}
	return int(math.Round(usage))
}

func readProcStat() (uint64, uint64, error) {
	file, err := os.Open("/proc/stat")
	if err != nil {
		return 0, 0, err
	}
	defer file.Close()

	scanner := bufio.NewScanner(file)
	if scanner.Scan() {
		line := scanner.Text()
		if strings.HasPrefix(line, "cpu ") {
			fields := strings.Fields(line)
			if len(fields) >= 5 {
				user, _ := strconv.ParseUint(fields[1], 10, 64)
				nice, _ := strconv.ParseUint(fields[2], 10, 64)
				system, _ := strconv.ParseUint(fields[3], 10, 64)
				idle, _ := strconv.ParseUint(fields[4], 10, 64)
				
				var iowait, irq, softirq, steal uint64
				if len(fields) >= 6 {
					iowait, _ = strconv.ParseUint(fields[5], 10, 64)
				}
				if len(fields) >= 7 {
					irq, _ = strconv.ParseUint(fields[6], 10, 64)
				}
				if len(fields) >= 8 {
					softirq, _ = strconv.ParseUint(fields[7], 10, 64)
				}
				if len(fields) >= 9 {
					steal, _ = strconv.ParseUint(fields[8], 10, 64)
				}

				total := user + nice + system + idle + iowait + irq + softirq + steal
				idleTotal := idle + iowait
				return total, idleTotal, nil
			}
		}
	}
	return 0, 0, fmt.Errorf("invalid format")
}

func getRAMUsage() (int, float64, float64) {
	// 1. Try to read cgroups v2 memory limit and usage first (LXC/Docker)
	currentBytes, err1 := readCgroupValue("/sys/fs/cgroup/memory.current")
	maxBytes, err2 := readCgroupValue("/sys/fs/cgroup/memory.max")

	if err1 == nil && err2 == nil && maxBytes > 0 {
		usedGB := float64(currentBytes) / (1024 * 1024 * 1024)
		totalGB := float64(maxBytes) / (1024 * 1024 * 1024)
		percent := int(math.Round((usedGB / totalGB) * 100))
		return percent, roundToTwoDecimals(usedGB), roundToTwoDecimals(totalGB)
	}

	// 2. Try cgroups v1 (fallback)
	currentBytesV1, err3 := readCgroupValue("/sys/fs/cgroup/memory/memory.usage_in_bytes")
	maxBytesV1, err4 := readCgroupValue("/sys/fs/cgroup/memory/memory.limit_in_bytes")
	if err3 == nil && err4 == nil && maxBytesV1 > 0 && maxBytesV1 < 9000000000000000000 {
		usedGB := float64(currentBytesV1) / (1024 * 1024 * 1024)
		totalGB := float64(maxBytesV1) / (1024 * 1024 * 1024)
		percent := int(math.Round((usedGB / totalGB) * 100))
		return percent, roundToTwoDecimals(usedGB), roundToTwoDecimals(totalGB)
	}

	// 3. Fallback to /proc/meminfo (general host system)
	memTotal, memAvail, errInfo := readProcMeminfo()
	if errInfo == nil && memTotal > 0 {
		totalGB := float64(memTotal) / (1024 * 1024)
		availGB := float64(memAvail) / (1024 * 1024)
		usedGB := totalGB - availGB
		percent := int(math.Round((usedGB / totalGB) * 100))
		return percent, roundToTwoDecimals(usedGB), roundToTwoDecimals(totalGB)
	}

	// 4. Default Mock values (e.g. Windows local development)
	return 14, 0.07, 0.50
}

func readCgroupValue(path string) (uint64, error) {
	content, err := os.ReadFile(path)
	if err != nil {
		return 0, err
	}
	valStr := strings.TrimSpace(string(content))
	if valStr == "max" {
		return 0, fmt.Errorf("cgroup limit is set to max")
	}
	return strconv.ParseUint(valStr, 10, 64)
}

func readProcMeminfo() (uint64, uint64, error) {
	file, err := os.Open("/proc/meminfo")
	if err != nil {
		return 0, 0, err
	}
	defer file.Close()

	var memTotal, memAvail uint64
	scanner := bufio.NewScanner(file)
	for scanner.Scan() {
		line := scanner.Text()
		if strings.HasPrefix(line, "MemTotal:") {
			fields := strings.Fields(line)
			if len(fields) >= 2 {
				memTotal, _ = strconv.ParseUint(fields[1], 10, 64)
			}
		} else if strings.HasPrefix(line, "MemAvailable:") {
			fields := strings.Fields(line)
			if len(fields) >= 2 {
				memAvail, _ = strconv.ParseUint(fields[1], 10, 64)
			}
		}
		if memTotal > 0 && memAvail > 0 {
			break
		}
	}
	if memTotal == 0 {
		return 0, 0, fmt.Errorf("could not parse total memory")
	}
	if memAvail == 0 {
		memAvail = memTotal / 2
	}
	return memTotal, memAvail, nil
}

func roundToTwoDecimals(val float64) float64 {
	return math.Round(val*100) / 100
}
