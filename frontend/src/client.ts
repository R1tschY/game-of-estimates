import { writable, Writable } from 'svelte/store'
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
    players: PlayerState[];
}

export interface PlayerJoinedEvent {
    player: PlayerState;
}

export interface PlayerChangedEvent {
    player: PlayerState;
}

export interface PlayerLeftEvent {
    player_id: string;
}

export interface GameChangedEvent {
    game_state: GameState;
}

export interface PlayerState {
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


export class Client {
    ws: WebSocket

    welcome: Signal<WelcomeMessageEvent> = new Signal();
    joined: Signal<JoinedEvent> = new Signal();
    playerJoined: Signal<PlayerJoinedEvent> = new Signal();
    playerChanged: Signal<PlayerChangedEvent> = new Signal();
    playerLeft: Signal<PlayerLeftEvent> = new Signal();
    stateChanged: Signal<GameChangedEvent> = new Signal();
    rejected: Signal<RejectedEvent> = new Signal();

    constructor(wsService: WebSocketService) {
        wsService.ws_store.subscribe(($ws) => (this.ws = $ws))
        wsService.message.connect((evt) => this.on_messageArrived(evt))
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
            type: 'SetVoter',
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

    set_name(name: string) {
        this._send({
            type: 'SetName',
            name
        })
    }

    joinRoom(room: string) {
        this._send({
            type: 'JoinRoom',
            room
        })
    }

    createRoom(deck: string) {
        this._send({
            type: 'CreateRoom',
            deck
        })
    }

    _send(payload: any) {
        if (this.ws) {
            this.ws.send(JSON.stringify(payload))
        }
    }

    on_messageArrived(event: any) {
        console.debug('Got message', event)
        switch (event.type) {
            case 'Welcome':
                this.welcome.emit(event as WelcomeMessageEvent)
                break
    
            case 'Joined':
                this.joined.emit(event as JoinedEvent)
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
        this.reconnectTimer = setTimeout(this.connect, reconnectTimeout)
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
