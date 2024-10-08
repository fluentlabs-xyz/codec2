# FLuentbase codec v2

## Codec macro

Codec macro allows you to derrive a codec for a struct.

### Solidity

- при кодировании структуры, у которой хотя бы одно из полей динамическое, нам нужно кодировать структуру следующим образом

| Index | Field          | Description                                     |
| ----- | -------------- | ----------------------------------------------- |
| 0     | uint256        | offset                                          |
| 1     | static fields  | all static fields, the same order as in struct  |
| 2     | dynamic fields | all dynamic fields, the same order as in struct |

для солидити смещение до данных всегда будет на 32 байта меньше для динамических структур - это происходит из-за того, что оффсет учитывается от начала данных, а не от смещения. Таким образом, чтобы получить правильное смещение, нам нужно добавить 32 байта к оффсету.
