import json
import re
from pathlib import Path
import pandas as pd
import seaborn as sns
import matplotlib.pyplot as plt
import numpy as np

CURR_DIR = Path(__file__).resolve().parent
ROOT = CURR_DIR.parent / "target" / "criterion"
NANO_TO_MILLI = 1_000_000

def extract_data():
    estimates_data = []

    # Traverse algorithm directories
    for algorithm_dir in ROOT.glob("bpe training_*b"):

        data_size_match = re.search(
            r"bpe training_(\d+)b",
            algorithm_dir.name
        )

        if not data_size_match:
            continue

        data_size = int(data_size_match.group(1))

        # Traverse naive_300, optimized_500, etc.
        for benchmark_dir in algorithm_dir.iterdir():

            if not benchmark_dir.is_dir():
                continue

            match = re.fullmatch(
                r"(optimizedstream|optimizedstore)_(\d+)",
                benchmark_dir.name
            )

            if not match:
                continue

            implementation = match.group(1)
            vocab_size = int(match.group(2))

            # Locate Criterion's new directory
            new_dir = benchmark_dir / "new"

            estimates_file = new_dir / "estimates.json"

            if not estimates_file.exists():
                print(f"Missing: {estimates_file}")
                continue

            # Load raw samples
            with estimates_file.open() as f:
                estimates = json.load(f)

            estimates_data.append({
                "implementation": implementation,
                "data_size": data_size,
                "vocab_size": vocab_size,
                "mean": estimates["mean"]["point_estimate"] / NANO_TO_MILLI,
                "median": estimates["median"]["point_estimate"] / NANO_TO_MILLI,
            })

    return estimates_data

def mean_exec_vs_vocab(estimate_df):
    sns.set_theme(style="whitegrid")

    sns.relplot(
        data=estimate_df,
        x="vocab_size",
        y="mean",
        hue="implementation",
        col="data_size",
        col_wrap=2,
        kind="line",
        marker="o",
        facet_kws={"sharey": False},
        height=4,
        aspect=1.2
    )

def mean_exec_vs_data(estimate_df):
    sns.set_theme(style="whitegrid")

    sns.relplot(
        data=estimate_df,
        x="data_size",
        y="mean",
        hue="implementation",
        col="vocab_size",
        col_wrap=2,
        kind="line",
        marker="o",
        facet_kws={"sharey": False},
        height=4,
        aspect=1.2
    )

def speedup(estimate_df):
    speedup_df = estimate_df.pivot(
        index=["data_size", "vocab_size"],
        columns="implementation",
        values="mean"
    ).reset_index()

    speedup_df["speedup"] = (
        speedup_df["optimizedstream"] / speedup_df["optimizedstore"]
    )

    sns.relplot(
        data=speedup_df,
        x="vocab_size",
        y="speedup",
        hue="data_size",
        kind="line",
        marker="o",
        height=5,
        aspect=1.4
    )

    sns.relplot(
        data=speedup_df,
        x="data_size",
        y="speedup",
        hue="vocab_size",
        kind="line",
        marker="o",
        height=5,
        aspect=1.4
    )
    
    plt.xscale('log')
    plt.axhline(1, color="black", linestyle="--")

def plot_3d_surface(estimate_df, implementation):
    data = estimate_df[
        estimate_df["implementation"] == implementation
    ]

    pivot = data.pivot(
        index="data_size",
        columns="vocab_size",
        values="mean"
    )

    X, Y = np.meshgrid(pivot.columns, pivot.index)
    Z = pivot.values

    fig = plt.figure(figsize=(11, 8))
    ax = fig.add_subplot(111, projection="3d")

    # X = vocabulary size
    # Y = training data size
    # Z = execution time

    surface = ax.plot_surface(
        X, Y, Z,
        cmap="viridis",
        edgecolor="black",
        alpha=0.85
    )

    ax.set_xlabel("Vocabulary Size")
    ax.set_ylabel("Training Data Size(bytes)")
    ax.set_zlabel("Execution Time (ms)")

    ax.set_title(f"{implementation.capitalize()} Performance")

    fig.colorbar(surface, ax=ax, label="Execution Time")

def main():
    estimate_data = extract_data()
    print(f"Extracted {len(estimate_data)} benchmarks")

    estimate_df = pd.DataFrame(estimate_data)

    print(estimate_df.head())
    print(estimate_df.info())

    plot_3d_surface(estimate_df, "optimizedstream")
    plot_3d_surface(estimate_df, "optimizedstore")
    mean_exec_vs_vocab(estimate_df)
    mean_exec_vs_data(estimate_df)
    speedup(estimate_df)
    
    plt.show()

if __name__ == '__main__':
    main()