import '../ext/string'

import '../main.sass'

import { mount } from 'svelte'
import CreateRoomForm from '../components/CreateRoomForm.svelte'

const createRoomForm = mount(CreateRoomForm, {
    target: document.getElementById('create-room-form')!,
})

export default createRoomForm
