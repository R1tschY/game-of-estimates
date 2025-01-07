import '../ext/string'

import { mount } from 'svelte'
import RoomContainer from '../components/RoomContainer.svelte'

const target = document.getElementById('room-container')!

const roomContainer = mount(RoomContainer, {
    target,
    props: {
        id: target.dataset.roomid!,
    },
})

export default roomContainer
