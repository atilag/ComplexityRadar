# Complexity Radar
[![License](https://img.shields.io/github/license/atilag/ComplexityRadar.svg?style=popout-square)](https://opensource.org/licenses/Apache-2.0)<!--- long-description-skip-begin -->[![Release](https://img.shields.io/github/release/atilag/ComplexityRadar.svg?style=popout-square)](https://github.com/atilag/ComplexityRadar/releases)<!--- long-description-skip-end -->

Complexity Radar is a tool that shows the most changed files in a GitHub repository and the complexity index for these files. It uses several techniques to measure complexity, like [Cognitive Complexity](https://www.sonarsource.com/docs/CognitiveComplexity.pdf) by G. Ann Campbell or Afferent/Efferent coupling.

This tool aims to help teams dealing with large-scale software projects have more educated decisions on where to focus on reducing the complexity of the codebase.

## Installation

```bash
cargo add complexity-radar
```

## Getting started

You can run the command line tool like:

```bash
complexity-radar -u <github user> -r <repository name> -t <github token> -n <top n files to show>
```

## License

[Apache License 2.0](LICENSE.txt)