# How to install the bb backend

For the next exercises we will need to have the barretenberg backend installed on your machine.

To install the command tools:
1. Install `bbup` the installation script by running this in your terminal:

    ```bash
    curl -L https://raw.githubusercontent.com/AztecProtocol/aztec-packages/master/barretenberg/cpp/installation/install | bash
    ```

2. Reload your terminal shell environment:

    macOS:
    ```bash
    source ~/.zshrc
    ```

    Linux:
    ```bash
    source ~/.bashrc
    ```

3. Install the version of `bb` compatible with your Noir version; here **Noir v0.34.0**:

    ```bash
    bbup -v 0.55.0
    ```

4. Check if the installation was successful:

    ```bash
    bb --version
    ```

If installation was successful, the command would print the version of `bb` installed.

(More information about the installation : https://github.com/AztecProtocol/aztec-packages/blob/master/barretenberg/cpp/src/barretenberg/bb/readme.md#installation)