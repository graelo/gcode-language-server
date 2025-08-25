# Instructions

In this folder, I would like us to create a lsp server that I can use with
neovim for instance.

In terms of communication with neovim, I'll need your opinion on whether to do
this with stdin or with a protocol, I'll favor whatever is standard.

I want to write it in rust.

I want it to support different gcode flavors (I'll start with the gcode flavor
of prusa -- by flavor I mean all the non-standard commands they use for their
firmware).

I want to provide developers with the ability to define their own flavor of
Gcode, for instance to support other printers, or to support evolutions of
Prusa GCode.

Prusa Gcode page is here: <https://help.prusa3d.com/article/buddy-firmware-specific-g-code-commands_633112>

If you agree, I think creating a PRD for this work could help you. Also be
prepared to store your plan and findings in markdown documents in the docs/work
folder.

Let's create the PRD, ask me any question you feel is necessary. We'll then
craft and architecture document for the server, and start implementation, using
different issues. You will write the issues in docs/issues/.
