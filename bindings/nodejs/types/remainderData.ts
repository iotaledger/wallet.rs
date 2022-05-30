import type { AddressTypes, OutputTypes } from '@iota/types'
import type { Segment } from './output'

export type RemainderData = {
    output: OutputTypes,
    chain: Segment[],
    address: AddressTypes
}
