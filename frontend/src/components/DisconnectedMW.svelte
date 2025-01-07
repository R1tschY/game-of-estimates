<script lang="ts">
    import { connected } from '../stores'
</script>

<div class="modal" class:is-active={!$connected}>
    <div class="modal-background"></div>
    <div class="modal-content reconnecting-content">
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 55">
            <circle cx="35" cy="30" r="5" fill="#333">
                <animate
                    attributeName="opacity"
                    dur="1s"
                    values="0;1;0"
                    repeatCount="indefinite"
                    begin="0.2"
                />
            </circle>
            <circle cx="50" cy="30" r="5" fill="#333">
                <animate
                    attributeName="opacity"
                    dur="1s"
                    values="0;1;0"
                    repeatCount="indefinite"
                    begin="0.3"
                />
            </circle>
            <circle cx="65" cy="30" r="5" fill="#333">
                <animate
                    attributeName="opacity"
                    dur="1s"
                    values="0;1;0"
                    repeatCount="indefinite"
                    begin="0.4"
                />
            </circle>
        </svg>

        <h3 class="title is-4" style="text-align: center">Connecting ...</h3>
    </div>
    <!--<button class="modal-close is-large" aria-label="close"></button>-->
</div>

<style>
    .reconnecting-content {
        width: 300px;
        padding: 20px;
    }
</style>
