import { derived, get, Readable, Writable, writable } from 'svelte/store'
import { navigate } from 'svelte-routing'
import { client, GameState, OwnPlayerState, PlayerInfo, wsService } from './client'
import type { Option } from './basetypes'
import { writableLocalStorage as writableLocalStorage, derivedWritable as derivedWritable, derivedWritableProperty } from './store-utils'

// state

export const connected: Readable<boolean> = wsService.connected_store
export const connecting: Readable<boolean> = wsService.connecting_store
export const player_id: Readable<Option<string>> = writable(null)


export interface PlayerSettings {
    name: Option<string>,
    voter: boolean,
    debug: boolean
}

export const ownPlayerState: Writable<PlayerSettings> = writableLocalStorage('goe-player-settings', {
    name: null,
    voter: true,
    debug: false
})
ownPlayerState.subscribe((value) => {
    console.log("state changed", value)
    client.updatePlayer(value.voter, value.name)
})

export const debug: Readable<boolean> = derived(ownPlayerState, (value) => value.debug)

export const voter: Writable<boolean> = writable(true)
voter.subscribe((value) => {
    ownPlayerState.update((state) => {
        state.voter = value
        return state
    })
})

// export const voter: Writable<boolean> = derivedWritableProperty(
//     ownPlayerState,
//     function() { return this.voter },
//     function(value) { this.voter = value; }
// )

export const name: Writable<Option<string>> = writable(null)
name.subscribe((value) => {
    ownPlayerState.update((state) => {
        state.name = value
        return state
    })
})

export const vote: Writable<Option<string>> = writable(null)
vote.subscribe((value) => client.vote(value))

export const creating_room: Writable<boolean> = writable(false)

// mutations

// actions

interface RoomState {
    id: Option<string>,
    last_error: Option<string>,
    players: PlayerInfo[],
    state: Option<GameState>,
}

function initRoomState(): RoomState {
    return {
        id: null,
        last_error: null,
        players: [],
        state: null,
    }
}

export const room: Readable<RoomState> = (function createRoomState() {
    const { subscribe, set, update } = writable(initRoomState())

    client.welcome.connect(evt => {
        update((state) => {
            if (state.id !== null) {
                client.joinRoom(state.id)
            }
            return state
        })
    })

    client.joined.connect(evt => {
        update((room) => {
            if (room.id !== evt.room)
                navigate('/room/' + evt.room)
            return {
                id: evt.room,
                last_error: null,
                players: evt.players,
                state: evt.state,
            }
        })
    })

    client.rejected.connect(evt => {
        navigate('/')
        update((room) => {
            let state = initRoomState()
            state.last_error = "room does not exist"
            return state
        })
    })

    client.playerJoined.connect(evt => {
        update((room) => {
            room.players.push(evt.player)
            if (evt.player.voter) {
                room.state.votes[evt.player.id] = null
            }
            return room
        })
    })

    client.playerChanged.connect(evt => {
        update((room) => {
            let index = room.players.findIndex((p) => p.id == evt.player.id)
            if (index !== -1) {
                room.players[index] = evt.player
            }
            return room
        })
    })

    client.playerLeft.connect(evt => {
        update((room) => {
            let pid = get(player_id)
            let index = room.players.findIndex((p) => p.id == pid)
            if (index !== -1) {
                room.players.splice(index, 1)
            }
            delete room.state.votes[evt.player_id]
            return room
        })
    })

    client.stateChanged.connect(evt => {
        update((room) => {
            room.state = evt.game_state
            return room
        })
    })

    return {
        subscribe
    }
})()
