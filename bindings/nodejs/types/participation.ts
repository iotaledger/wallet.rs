import type { OutputId } from './output';

export interface ParticipationOverview {
    participations: Participations
}

export interface Participations {
    [eventId: ParticipationEventId]: {
        [outputId: OutputId]: TrackedParticipationOverview;
    };
};

export interface TrackedParticipationOverview {
    amount: string;
    answers: number[];
    blockId: string;
    endMilestoneIndex: number;
    startMilestoneIndex: number;
}

export interface ParticipationEvent {
    id: ParticipationEventId;
    data: ParticipationEventData;
}

export type ParticipationEventId = string;

export interface ParticipationEventStatus {
    milestoneIndex: number;
    status: string;
    questions?: QuestionStatus[];
    checksum: string;
}

export interface ParticipationEventData {
    name: string;
    milestoneIndexCommence: number;
    milestoneIndexStart: number;
    milestoneIndexEnd: number;
    payload: ParticipationEventPayload;
    additionalInfo: string;
}

export type ParticipationEventPayload = VotingEventPayload | StakingEventPayload;

export interface VotingEventPayload {
    type: ParticipationEventType.Voting;
    questions: Question[];
}

export interface StakingEventPayload {
    type: ParticipationEventType.Staking;
    text: string;
    symbol: string;
    numerator: string;
    denominator: string;
    requiredMinimumRewards: string;
    additionalInfo: string;
}

export interface Question {
    text: string;
    answers: Answer[];
    additionalInfo: string;
}

export interface Answer {
    value: number;
    text: string;
    additionalInfo: string;
}

export interface QuestionStatus {
    answers: AnswerStatus[]
}

export interface AnswerStatus {
    value: number;
    current: number;
    accumulated: number;
}

export enum ParticipationEventType {
    Voting = 0,
    Staking = 1,
}
