import type { OutputId } from './output'

export interface ParticipationOverview {
    participations: [EventId, [OutputId, TrackedParticipationOverview]]
}

export interface TrackedParticipationOverview {
    blockId: string
    amount: string
    startMilestoneIndex: number
    endMilestoneIndex: number
}

type EventId = string
