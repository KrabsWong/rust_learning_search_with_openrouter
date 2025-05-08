# LLM Search CLI

This CLI tool leverages the power of Large Language Models (LLMs) and web search to provide comprehensive answers to user queries. It follows a three-phase process:

1.  **Keyword Generation**: Uses an LLM (via OpenRouter) to generate relevant search keywords based on the user's input query.
2.  **Web Search**: Fetches search results using the Exa API based on the generated keywords.
3.  **Final Answer Generation**: Combines the user's original query and the summarized search results, then uses another LLM (via OpenRouter) to generate a final, comprehensive answer.

## Features

*   **Intelligent Keyword Generation**: Dynamically creates effective search terms.
*   **Exa Integration**: Utilizes Exa for robust web searching capabilities.
*   **OpenRouter Integration**: Leverages various LLMs for keyword generation and final answer synthesis.
*   **Streaming Output**: The final answer from OpenRouter is streamed to the console for a better user experience.
*   **Token Usage Display**: Shows token consumption for OpenRouter API calls, helping to monitor costs.
*   **Interactive Input**: Prompts the user for their query.
*   **Styled Console Output**: Uses colored and styled text for better readability of different phases and information.


Click image below to watch the video:

<img width="500" alt="Terminal Screenshot part 1" src="https://github.com/user-attachments/assets/2703f05d-e7d5-4ec9-a43b-15a6c4103fe5" />

<img width="500" alt="Terminal Screenshot part 2" src="https://github.com/user-attachments/assets/17ebcacf-f947-4eb0-8d2d-0be23ea2125f" />

[![Using large models and Exa to fetch data in the command line.](https://github.com/user-attachments/assets/260ac623-94ed-4b55-a011-c488415dc27e)](https://www.youtube.com/shorts/fPbFh3uxyVg)

## Setup

### Prerequisites

*   Rust programming language and Cargo (Rust's package manager). You can install them from [rust-lang.org](https://www.rust-lang.org/tools/install).

### Configuration

1.  **Clone the repository (if applicable) or ensure you are in the project's root directory.**

2.  **Create a `.env` file** in the root of the project directory. This file will store your API keys.

3.  **Add your API keys to the `.env` file:**

    ```env
    OPENROUTER_API_KEY="your_openrouter_api_key_here"
    EXA_API_KEY="your_exa_api_key_here"
    ```

    Replace `your_openrouter_api_key_here` and `your_exa_api_key_here` with your actual API keys from [OpenRouter.ai](https://openrouter.ai/) and [Exa.ai](https://exa.ai/) respectively.

### Build

Navigate to the project's root directory in your terminal and build the project using Cargo:

```bash
cargo build --release
```

This will create an optimized executable in the `target/release/` directory.

## Usage

After building the project, you can run the CLI tool from the project's root directory:

```bash
cargo run
```

Alternatively, you can run the compiled binary directly:

```bash
./target/release/rust_learning_search_with_openrouter
```

(The binary name might be different based on your `Cargo.toml` `name` field, typically it's the project directory name if not specified otherwise).

The tool will then prompt you to input your query:

```
Please input what you want in the next line...
> your search query here
```

Enter your query and press Enter. The tool will then proceed through the three phases: generating keywords, fetching search results, and generating the final answer, displaying progress and information along the way.

## Project Structure

*   `src/main.rs`: Main application logic, orchestrates the workflow.
*   `src/openrouter_client.rs`: Handles interactions with the OpenRouter API (keyword generation, final answer).
*   `src/exa_client.rs`: Handles interactions with the Exa API (fetching search results).
*   `src/models.rs`: Defines data structures (structs) for API requests and responses.
*   `src/utils.rs`: Utility functions (e.g., creating spinners for progress indication).
*   `.env` (you create this): Stores API keys.
*   `.gitignore`: Specifies intentionally untracked files that Git should ignore.
*   `Cargo.toml`: Rust package manifest, defines project metadata and dependencies.
*   `README.md`: This file.
