<script lang="ts">
    import { page } from '$app/state'
    import RoomContainer from '$/components/organisms/RoomContainer.svelte'

    let id = page.data.id
</script>

<RoomContainer {id}></RoomContainer>
