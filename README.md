# ASSDraw

**ASSDraw** is a tool designed to create and edit vector shapes for use in **ASS (Advanced SubStation Alpha)** subtitle files. It is built with C++ and utilizes the AGG (Anti-Grain Geometry) library to handle graphical drawing operations. The tool provides an intuitive interface to generate precise and complex shapes which can then be exported to the ASS subtitle format.

## Features
- **Vector Drawing**: Create shapes like rectangles, circles, and lines, which are often used in ASS subtitle styling.
- **ASS Export**: Direct export of the created shapes into the ASS subtitle format, for use with subtitle editing software.
- **Customizable Interface**: Integrated with wxWidgets, allowing for flexible user interface adjustments.
- **Cross-Platform**: Designed to be compatible with various platforms, ensuring versatility.

## Installation

1. Clone the repository to your local machine:
    ```bash
    git clone https://github.com/KerimDemirkaynak/ASSDraw.git
    ```

2. Ensure you have all necessary dependencies:
    - AGG (Anti-Grain Geometry): A high-quality graphics library used for rendering vector graphics.
    - wxWidgets: A C++ library used for building graphical user interfaces.

3. To build the project, you’ll need a C++ compiler (like GCC) and a build system (like Make or CMake). Follow the standard build instructions for your platform:
    ```bash
    make
    ```

## Usage

Once the project is compiled:

1. Run the **ASSDraw** executable.
2. Use the graphical interface to design your shapes and adjust their properties.
3. Export the shapes into an ASS subtitle file that can be used for styling subtitles.

## Dependencies
- **wxWidgets** (v3.0 or later)
- **AGG Library** (v2.5)
- C++11 or later

## Building From Source

### Windows:
- Download and install **wxWidgets** and **AGG**.
- Set the appropriate environment variables for your compiler.
- Run the build command, e.g., `make` or use your IDE's build tools.

### Linux/macOS:
- Install the necessary packages via your package manager.
    ```bash
    sudo apt-get install libwxgtk3.0-dev libagg-dev
    ```
- Run the build command.

## Configuration

The project uses a **Default Profile** which configures paths for libraries and output files. You can customize the settings by modifying the `Default Profile` configuration or creating a new one.

```plaintext
[Profile1]
Include Paths: /path/to/libs
Output Paths: /path/to/output
```

## License

ASSDraw is licensed under the **MIT License**. See [LICENSE](LICENSE) for more information.

## Contributing

If you want to contribute, feel free to fork the repository, make changes, and submit a pull request. Please ensure you follow the guidelines for coding style and include tests where applicable.

## Acknowledgments
- **AGG** Library for providing the rendering capabilities.
- **wxWidgets** for cross-platform GUI support.
