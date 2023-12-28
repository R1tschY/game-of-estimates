import type { Readable, Writable } from 'svelte/store'
import { writable } from 'svelte/store'
import type { Option } from './basetypes'
import { Signal } from './events'

// store

// consts

const reconnectTimeout = 5000

// client

export interface BaseMessageEvent {
    type: string
}

export interface WelcomeMessageEvent extends BaseMessageEvent {
    type: 'Welcome'
    player_id: string
}

export interface RejectedEvent extends BaseMessageEvent {
    type: 'Rejected'
}

export interface JoinedEvent extends BaseMessageEvent {
    type: 'Joined'
    room: string
    state: GameState
    players: PlayerInfo[]
}

export interface PlayerJoinedEvent extends BaseMessageEvent {
    type: 'PlayerJoined'
    player: PlayerInfo
}

export interface PlayerChangedEvent extends BaseMessageEvent {
    type: 'PlayerChanged'
    player: PlayerInfo
}

export interface PlayerLeftEvent extends BaseMessageEvent {
    type: 'PlayerLeft'
    player_id: string
}

export interface GameChangedEvent extends BaseMessageEvent {
    type: 'GameChanged'
    game_state: GameState
}

export interface PlayerInfo {
    id: string
    name: Option<string>
    voter: boolean
}

export interface OwnPlayerState {
    name: Option<string>
    voter: boolean
}

export interface GameState {
    deck: string
    open: boolean
    votes: Record<string, Option<string>>
}

export type PlayerState = 'connecting' | 'outside' | 'joining' | 'joined'

export class Client {
    _ws!: Option<WebSocket>

    state: Writable<PlayerState>
    playerId: Writable<Option<string>>
    roomId: Writable<Option<string>>
    lastError: Writable<Option<string>>

    welcome: Signal<WelcomeMessageEvent> = new Signal()
    joined: Signal<JoinedEvent> = new Signal()
    playerJoined: Signal<PlayerJoinedEvent> = new Signal()
    playerChanged: Signal<PlayerChangedEvent> = new Signal()
    playerLeft: Signal<PlayerLeftEvent> = new Signal()
    stateChanged: Signal<GameChangedEvent> = new Signal()
    rejected: Signal<RejectedEvent> = new Signal()

    constructor(wsService: WebSocketService) {
        this.state = writable('connecting')
        this.playerId = writable(null)
        this.roomId = writable(null)
        this.lastError = writable(null)

        wsService.ws_store.subscribe(($ws) => (this._ws = $ws))
        wsService.message.connect((evt) => this._onMessageArrived(evt))
        wsService.disconnected.connect(() => this._onDisconnected())
    }

    updatePlayer(voter: boolean, name: Option<string>) {
        this._send({
            type: 'UpdatePlayer',
            voter,
            name,
        })
    }

    vote(vote: Option<string>) {
        this._send({
            type: 'Vote',
            vote,
        })
    }

    forceOpen() {
        this._send({
            type: 'ForceOpen',
        })
    }

    restart() {
        this._send({
            type: 'Restart',
        })
    }

    setName(name: string) {
        this._send({
            type: 'SetName',
            name,
        })
    }

    joinRoom(room: string) {
        this.state.set('joining')
        this.roomId.set(room)
        this._send({
            type: 'JoinRoom',
            room,
        })
    }

    createRoom(deck: string) {
        this.state.set('joining')
        this._send({
            type: 'CreateRoom',
            deck,
        })
    }

    _send(payload: object) {
        if (this._ws) {
            this._ws.send(JSON.stringify(payload))
        }
    }

    private _onDisconnected(): void {
        this.state.set('connecting')
        this.playerId.set(null)
    }

    private _onMessageArrived(event: BaseMessageEvent) {
        console.debug('Got message', event)
        switch (event.type) {
            case 'Welcome': {
                this.state.set('outside')

                const welcomeEvt = event as WelcomeMessageEvent
                this.playerId.set(welcomeEvt.player_id)
                this.welcome.emit(welcomeEvt)
                break
            }

            case 'Joined': {
                this.state.set('joined')

                const joinedEvt = event as JoinedEvent
                this.roomId.set(joinedEvt.room)
                this.joined.emit(joinedEvt)
                break
            }

            case 'PlayerJoined':
                this.playerJoined.emit(event as PlayerJoinedEvent)
                break

            case 'PlayerChanged':
                this.playerChanged.emit(event as PlayerChangedEvent)
                break

            case 'PlayerLeft':
                this.playerLeft.emit(event as PlayerLeftEvent)
                break

            case 'GameChanged':
                this.stateChanged.emit(event as GameChangedEvent)
                break

            case 'Rejected':
                this.state.set('outside')
                this.roomId.set(null)
                this.lastError.set('Room does not exist')
                this.rejected.emit(event as RejectedEvent)
                break

            default:
                console.error('Unknown message', event)
                break
        }
    }
}

export class WebSocketService {
    ws!: Option<WebSocket>
    ws_store: Writable<Option<WebSocket>>
    connecting_store: Writable<boolean>
    connected_store: Writable<boolean>
    error_store: Writable<boolean>
    reconnectTimer: Option<number>

    message: Signal<BaseMessageEvent> = new Signal()
    connected: Signal<undefined> = new Signal()
    disconnected: Signal<undefined> = new Signal()
    error: Signal<undefined> = new Signal()

    constructor() {
        this.ws_store = writable(null)
        this.connecting_store = writable(true)
        this.connected_store = writable(false)
        this.error_store = writable(false)
        this.reconnectTimer = null
        this.connect()
    }

    clearReconnectTimer() {
        if (this.reconnectTimer !== null) {
            clearTimeout(this.reconnectTimer)
            this.reconnectTimer = null
        }
    }

    startReconnectTimer() {
        this.clearReconnectTimer()
        this.reconnectTimer = Number(
            setTimeout(() => this.connect(), reconnectTimeout),
        )
    }

    on_connected(event: Event) {
        console.log('connected', event)
        this.connected_store.set(true)
        this.connecting_store.set(false)
        this.error_store.set(false)
        this.ws_store.set(this.ws)
        this.connected.emit(undefined)
        this.clearReconnectTimer()
    }

    on_disconnected(event: CloseEvent) {
        console.log('disconnected', event)
        this.connecting_store.set(false)
        this.connected_store.set(false)
        this.disconnected.emit(undefined)
        this.startReconnectTimer()
    }

    on_connection_error(event: Event) {
        console.log('error', event)
        this.connected_store.set(false)
        this.connecting_store.set(false)
        this.error_store.set(true)
        this.error.emit(undefined)
        this.startReconnectTimer()
    }

    connect() {
        const url = import.meta.env.GOE_WEBSOCKET_URL || this.guessWsAddr()
        console.debug('connecting to ' + url + ' ...', url)
        this.connecting_store.set(true)

        this.ws = new WebSocket(url)
        this.ws.addEventListener('open', (evt) => this.on_connected(evt))
        this.ws.addEventListener('message', (evt) => {
            this.message.emit(JSON.parse(evt.data))
        })
        this.ws.addEventListener('close', (evt) => this.on_disconnected(evt))
        this.ws.addEventListener('error', (evt) =>
            this.on_connection_error(evt),
        )
    }

    private guessWsAddr(): string {
        const loc = window.location
        const protocol = loc.protocol === 'http:' ? 'ws:' : 'wss:'
        return `${protocol}//${loc.host}/ws`
    }
}

export const wsService: WebSocketService = new WebSocketService()

export const client: Client = new Client(wsService)
export const playerState: Readable<PlayerState> = client.state
