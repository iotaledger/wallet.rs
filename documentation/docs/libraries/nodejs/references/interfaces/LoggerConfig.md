# Interface: LoggerConfig

Logger output configuration.

## Table of contents

### Properties

- [name](LoggerConfig.md#name)
- [levelFilter](LoggerConfig.md#levelfilter)
- [targetFilter](LoggerConfig.md#targetfilter)
- [targetExclusions](LoggerConfig.md#targetexclusions)
- [colorEnabled](LoggerConfig.md#colorenabled)

## Properties

### name

• `Optional` **name**: `string`

Name of an output file, or `stdout` for standard output.

___

### levelFilter

• `Optional` **levelFilter**: ``"error"`` \| ``"off"`` \| ``"warn"`` \| ``"info"`` \| ``"debug"`` \| ``"trace"``

Log level filter of an output.

___

### targetFilter

• `Optional` **targetFilter**: `string`[]

Log target filters of an output.

___

### targetExclusions

• `Optional` **targetExclusions**: `string`[]

Log target exclusions of an output.

___

### colorEnabled

• `Optional` **colorEnabled**: `boolean`

Color flag of an output.
