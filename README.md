# Plat - A local template management tool

Plat is a local template management tool that allows you to use locally stored templates on the go. It's a command line tool so it requires environment variables to be set up.

## Installation

- Download the latest release from the [releases page](https://github.com/someramsey/plat/releases/latest)
- Put the downloaded file somewhere
- Add the path to the downloaded file to your PATH environment variable

## Usage

- Link the current directory as a template: `plat link <template-name?>`
- Load a template: `plat load <template-name>`
- Unlink the current directory: `plat unlink`
- List all linked templates: `plat list`


When linking a template you will be prompted to enter a name, the name must be unique because it works as an identifier for the template.

https://github.com/user-attachments/assets/212b8cee-815f-41f1-a30b-bb2416ebeeb8

