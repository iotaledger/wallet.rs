<<<<<<< HEAD
// import type { OutputId } from './output';

export interface ParticipationOverview {
    participations: {[eventId: string]: { [outputId: string]: TrackedParticipationOverview }};
}

export interface TrackedParticipationOverview {
    blockId: string;
    amount: string;
    startMilestoneIndex: number;
    endMilestoneIndex: number;
}

export interface Event {
    id: EventId;
    data: EventData;
}

export type EventId = string;

export interface EventStatus {
    milestoneIndex: number;
    status: string;
    questions?: Answer[];
    checksum: string;
}

export interface EventData {
=======
import type { Node } from './network';
import type { OutputId } from './output';

export interface ParticipationOverview {
    participations: Participations;
}

export interface Participations {
    [eventId: ParticipationEventId]: {
        [outputId: OutputId]: TrackedParticipationOverview;
    };
}

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

export interface ParticipationEventRegistrationOptions {
    node: Node;
    eventsToRegister?: ParticipationEventId[];
    eventsToIgnore?: ParticipationEventId[];
}

export interface ParticipationEventWithNodes {
    id: ParticipationEventId;
    data: ParticipationEventData;
    nodes: Node[];
}

export type ParticipationEventId = string;

export type ParticipationEventMap = {
    [id: ParticipationEventId]: ParticipationEventWithNodes
}

export interface ParticipationEventStatus {
    milestoneIndex: number;
    status: string;
    questions?: QuestionStatus[];
    checksum: string;
}

export interface ParticipationEventData {
>>>>>>> 5d1939575223b8004d642a02018d3e65f2ec4dbf
    name: string;
    milestoneIndexCommence: number;
    milestoneIndexStart: number;
    milestoneIndexEnd: number;
<<<<<<< HEAD
    payload: EventPayload;
    additionalInfo: string;
}

export type EventPayload = VotingEventPayload | StakingEventPayload;
=======
    payload: ParticipationEventPayload;
    additionalInfo: string;
}

export type ParticipationEventPayload =
    | VotingEventPayload
    | StakingEventPayload;
>>>>>>> 5d1939575223b8004d642a02018d3e65f2ec4dbf

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

<<<<<<< HEAD
=======
export interface QuestionStatus {
    answers: AnswerStatus[];
}

export interface AnswerStatus {
    value: number;
    current: number;
    accumulated: number;
}

>>>>>>> 5d1939575223b8004d642a02018d3e65f2ec4dbf
export enum ParticipationEventType {
    Voting = 0,
    Staking = 1,
}
