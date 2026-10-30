# Chord Generator

This is a basic rust terminal application that gives the user the ability to create a `.wav` file with a chord of their choosing.

The user must specify the wave format used: sine, saw, or square.

## CLI

### Usage

#### Installing the binary

`cd cli`
`cargo install --path .`
The run all commands from `cli`.

#### Wave types

`chord-generator generate sine Bb D F`

`chord-generator generate saw Bb D F`

`chord-generator generate square Bb D F`

#### Generate a chord

The `generate` command has the capability to build chords from C4 to C5 (chromatically inclusive).
The user must use the flat enharmonic of each note if an accidental is involved (e.g. F# is Gb).
The user can specify whether they would like the note built from middle C (C4) as a base, or whether that note should be up the octave (notated by prefixing the note with a "u").

##### Examples

`chord-generator generate sine A C E` (note how middle C is lower than A and E)

![Piano diagram with the notes A, C and E highlighted](assets/ace.png)

`chord-generator generate sine Gb Db Bb`

![Piano diagram with the notes Gb, Db, and Bb highlighted](assets/gbdbbb.png)

`chord-generator generate sine F Ab uC uEb`

![Piano diagram with the notes F, Ab, C, and Eb highlighted](assets/fabceb.png)

#### Morph between two chords

`chord-generator morph square Db uE Ab Gb --to F uA uG Eb --duration 10`

## Server

Navigate into the `server` directory and run `cargo run` which will start a server on `localhost:3000`.

### Generate a chord

To generate a chord, go to `localhost:3000/generate` and ensure you pass in the query params needed.

e.g. "http://localhost:3000/generate?wave_type=Sine&notes=C,E,G&duration=2.5"

Currently, this will create a WAV file locally, but functionality will soon exist in future to play the WAV file in browser or choose to download it.
