# Complexity Radar

Complexity Radar is a tool that shows the most changed files in a GitHub repository and the complexity index for these files. It uses several techniques to measure complexity, like [Cognitive Complexity](https://www.sonarsource.com/docs/CognitiveComplexity.pdf) by G. Ann Campbell or Afferent/Efferent coupling.

This tool aims to help teams dealing with large-scale software projects have more educated decisions on where to focus on reducing the complexity of the codebase.

[WARNING]
Calculating complexities is still in development, and only works for Rust projects :)


## Getting started

You can run the command line tool like:

```bash
complexity-radar -u <github user> -r <repository name> -t <github token> -n <top n files to show> --heat-map-only
```

## Dual License

This project is released under both the [Apache 2.0 License](LICENSE.Apache2) and the [MIT License](LICENSE.MIT). Users may choose to use either license, depending on their needs and preferences.