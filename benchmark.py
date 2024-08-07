from sys import exit as sys_exit
from sys import stderr as sys_stderr
from os.path import abspath as os_path_abspath
from os.path import dirname as os_path_dirname
from os.path import exists as os_path_exists
from os.path import isfile as os_path_isfile
from os.path import isdir as os_path_isdir
from os.path import basename as os_path_basename
from argparse import ArgumentParser
from argparse import Namespace
from time import time as time_time
from subprocess import run as subprocess_run
from subprocess import DEVNULL as subprocess_devnull


def main(executable_path: str, repo_path: str, iterations: int) -> int:
    if iterations < 1:
        print("ERROR: iteration number must be at least 1!", file=sys_stderr)
        return -1

    if not os_path_exists(f"{executable_path}"):
        print(
            f"ERROR: executable {executable_path} not found!", file=sys_stderr)
        return -1
    elif not os_path_isfile(executable_path):
        print(f"ERROR: {executable_path} is not a file!", file=sys_stderr)
        return -1
    elif (os_path_basename(executable_path) != "trace"
          and os_path_basename(executable_path) != "trace.exe"):
        print(f"ERROR: {os_path_basename(executable_path)} does not seem to "
              "be the 'trace' executable!", file=sys_stderr)
        return -1

    if not os_path_exists(f"{repo_path}"):
        print("ERROR: Target repo for benchmarking not found!",
              file=sys_stderr)
        return -1
    elif not os_path_isdir(repo_path):
        print("ERROR: Target repo for benchmarking is not a directory!",
              file=sys_stderr)
        return -1

    print(f"Beginning {iterations} iterations for benchmarking! "
          "Note that normal printed output will be suppressed.")

    total_time: float = 0

    for _ in range(iterations):
        start_time: float = time_time()
        subprocess_run([executable_path, "-d", repo_path],
                       stdout=subprocess_devnull)
        duration: float = time_time() - start_time
        total_time += duration

    print(
        "Average execution wall time over "
        f"{iterations} iterations: {1000*(total_time/iterations)}ms")

    return 0


if __name__ == "__main__":
    script_directory: str = os_path_dirname(os_path_abspath(__file__))

    parser: ArgumentParser = ArgumentParser(description="Benchmark Args")
    parser.add_argument("-e", "--exe", required=True, type=str)

    parser.add_argument("-r", "--repo", required=True, type=str)

    parser.add_argument("-i", "--iterations", required=True, type=int)

    args: Namespace = parser.parse_args()

    result: int = main(args.exe, args.repo, args.iterations)
    sys_exit(result)
