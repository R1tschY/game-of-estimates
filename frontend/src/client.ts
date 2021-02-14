import { readable, Readable, writable, Writable } from 'svelte/store'
import type { Option } from './basetypes'
import { Signal } from './events'

// store

// consts

const reconnectTimeout: number = 5000

declare var process: {
    env: {
        NODE_ENV: 'development' | 'production'
        GOE_WEBSOCKET_URL: string
    }
}

// client

export interface WelcomeMessageEvent {
    player_id: string,
}

export interface RejectedEvent { }

export interface JoinedEvent {
    room: string;
    state: GameState;
    players: PlayerInfo[];
}

export interface PlayerJoinedEvent {
    player: PlayerInfo;
}

export interface PlayerChangedEvent {
    player: PlayerInfo;
}

export interface PlayerLeftEvent {
    player_id: string;
}

export interface GameChangedEvent {
    game_state: GameState;
}

export interface PlayerInfo {
    id: string,
    name: Option<string>,
    voter: boolean,
}

export interface OwnPlayerState {
    name: Option<string>,
    voter: boolean,
}

export interface GameState {
    deck: string,
    open: boolean,
    votes: Record<string, Option<String>>,
}

export interface RejectedEvent { }

export type PlayerState = "connecting" | "outside" | "joining" | "joined"

export class Client {
    _ws: WebSocket

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
        this.state = writable("connecting")
        this.playerId = writable(null)
        this.roomId = writable(null)
        this.lastError = writable(null)

        wsService.ws_store.subscribe(($ws) => (this._ws = $ws))
        wsService.message.connect((evt) => this._onMessageArrived(evt))
        wsService.disconnected.connect((evt) => this._onDisconnected(evt))
    }

    updatePlayer(voter: boolean, name: Option<string>) {
        this._send({
            type: 'UpdatePlayer',
            voter,
            name
        })
    }

    vote(vote: Option<string>) {
        this._send({
            type: 'Vote',
            vote
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
            name
        })
    }

    joinRoom(room: string) {
        this.state.set("joining")
        this.roomId.set(room)
        this._send({
            type: 'JoinRoom',
            room
        })
    }

    createRoom(deck: string) {
        this.state.set("joining")
        this._send({
            type: 'CreateRoom',
            deck
        })
    }

    _send(payload: any) {
        setTimeout(() => {
            if (this._ws) {
                this._ws.send(JSON.stringify(payload))
            }
        }, 1000);
    }

    private _onDisconnected(evt: Event): void {
        this.state.set("connecting")
        this.playerId.set(null)
    }

    private _onMessageArrived(event: any) {
        console.debug('Got message', event)
        switch (event.type) {
            case 'Welcome':
                this.state.set("outside")

                const welcomeEvt = (event as WelcomeMessageEvent)
                this.playerId.set(welcomeEvt.player_id)
                this.welcome.emit(welcomeEvt)
                break
    
            case 'Joined':
                this.state.set("joined")

                const joinedEvt = (event as JoinedEvent)
                this.roomId.set(joinedEvt.room)
                this.joined.emit(joinedEvt)
                break
    
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
                this.state.set("outside")
                this.roomId.set(null)
                this.lastError.set("Room does not exist")
                this.rejected.emit(event as RejectedEvent)
                break
    
            default:
                console.error('Unknown message', event)
                break
        }
    }
}

export class WebSocketService {
    ws: Option<WebSocket>
    ws_store: Writable<Option<WebSocket>>
    connecting_store: Writable<boolean>
    connected_store: Writable<boolean>
    error_store: Writable<boolean>
    reconnectTimer: Option<number>

    message: Signal<any> = new Signal();
    connected: Signal<undefined> = new Signal();
    disconnected: Signal<undefined> = new Signal();
    error: Signal<undefined> = new Signal();

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
        this.reconnectTimer = setTimeout(() => this.connect(), reconnectTimeout)
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
        let url = process.env.GOE_WEBSOCKET_URL || 'ws://localhost:5500'
        console.debug('connecting to ' + url + ' ...')
        this.connecting_store.set(true)

        this.ws = new WebSocket(url)
        this.ws.addEventListener('open', evt => this.on_connected(evt))
        this.ws.addEventListener('message', (evt) => {
            this.message.emit(JSON.parse(evt.data))
        })
        this.ws.addEventListener('close', evt => this.on_disconnected(evt))
        this.ws.addEventListener('error', evt => this.on_connection_error(evt))
    }
}

export var wsService: WebSocketService = new WebSocketService()

export var client: Client = new Client(wsService)
export var playerState: Readable<PlayerState> = client.state

playerState.subscribe((value) => console.debug("Player state is now", value))
wsService.message.connect((evt) => console.debug("MSG", JSON.stringify(evt)))
