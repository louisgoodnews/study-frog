# StudyFrog 1.1.1

A patch release. The application is the same as 1.1.0; the change is in the continuous-integration setup.

## Fixed

- The release build failed on the CI runner: the linker ran out of resources while linking the debug test binaries. The dev and test profiles now build without debug info in the CI and release workflows. The application code is unchanged.

## Install

Download `StudyFrog-1.1.1-x86_64.AppImage` from this release, check it against the SHA-256 printed next to it, and start it from a desktop session.

## Upgrading from 1.1.0

No changes for users. See `RELEASE_NOTES_1.1.0.md` for the redesign, the choice questions and the setup wizard, and for the known limits, which still apply.
