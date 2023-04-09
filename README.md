ChatGPT CLI
===========

A command-line interface (CLI) for interacting with OpenAI's ChatGPT using the `chatgpt_rs` library.

Features
--------

*   Send messages and receive responses from ChatGPT in a conversational manner.
*   Save, load, and manage conversation history.
*   Supports various commands for managing conversations, such as `flush`, `save`, `load`, and `clear`.

Requirements
------------

*   Rust 1.56.0 or later
*   An OpenAI API key

Installation
------------

1.  Clone the repository:

    git clone https://github.com/your_username/chatgpt_cli.git

3.  Change to the project directory:

    cd chatgpt_cli

5.  Compile the project:

    cargo build --release

7.  (Optional) Add the compiled binary to your system's PATH for easier access.

Usage
-----

To run the ChatGPT CLI, execute the following command, replacing `your_api_key` with your OpenAI API key:

    ./target/release/chatgpt_cli your_api_key

You can interact with ChatGPT by typing messages directly in the CLI. The following commands are also available:

*   `flush`: Clears the current conversation.
*   `save`: Saves the current conversation to a JSON file. You'll be prompted for a file name.
*   `load`: Loads a saved conversation from a JSON file. You'll be prompted for a file name.
*   `clear`: Removes all saved conversations.

License
-------

This project is licensed under the [MIT License](LICENSE).
