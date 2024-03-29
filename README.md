﻿# CPU Usage Logger

CPU Usage Logger is a simple utility program that logs high CPU usage and tracks the CPU usage of processes on your system. The program was created by Rubin Raithel (@Coronon) and can be downloaded from the [GitHub repository](https://github.com/Coronon/cpu-usage-logger).

## Features

- Logs spikes in total CPU usage and CPU usage of individual processes
- Supports both CLI and logging to a file
- Allows customization of thresholds and measurement parameters

## Usage

The program can be run using the following command:

```sh
cpu-usage-logger [OPTIONS]
```

The available options are:

- `-b, --time-between-measurements`: How long to wait between measurements in seconds (default: 5)
- `-m, --measurement-time`: How long to measure for in seconds (CPU usage is an average over this time) (default: 1)
- `-t, --total-log-threshold`: Threshold of total CPU usage to start logging at in percent (default: 30)
- `-p, --process-log-threshold`: Threshold of single process CPU usage to start logging at in percent (default: 15)
- `-n, --number-of-processes-to-show`: Number of top CPU consuming processes to log when `total_log_threshold` is exceeded and to show in the CLI (default: 5)
- `-c, --cli`: CLI mode - periodically write stats to stdout
- `-l, --log-file`: Path to log file
- `-h, --help`: Print help
- `-V, --version`: Print version

## Example

To start logging CPU usage with the default settings, simply run the following command:

```sh
cpu-usage-logger
```

To log CPU usage to a file, use the following command:

```sh
cpu-usage-logger -l mylog.txt
```

To run the program in CLI mode, use the following command:

```sh
cpu-usage-logger -c
```

## Contributions

Contributions to the program are welcome. If you encounter any issues or have suggestions for improvement, please submit them to the [GitHub repository](https://github.com/Coronon/cpu-usage-logger/issues).

## License

The program is licensed under the [MIT License](https://github.com/Coronon/cpu-usage-logger/blob/master/LICENSE).
