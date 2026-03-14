<script lang="ts">
    import { page } from '$app/state'
    import RoomContainer from '../../components/RoomContainer.svelte'

    let id = page.data.id
</script>

<RoomContainer {id}></RoomContainer>
