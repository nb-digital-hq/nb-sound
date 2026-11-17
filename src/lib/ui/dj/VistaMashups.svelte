<!--
  @component
  DJ Mashups (ADR-018): el mashup que se ve (el que suena, el recién
  pedido o uno del historial) y, debajo, el formulario para pedir otro. Ocupa el
  mismo sitio que el DJ Privado y se carga solo cuando se abre.
-->
<script lang="ts">
  import MashupActivo from '#lib/ui/dj/MashupActivo.svelte';
  import PedirMashup from '#lib/ui/dj/PedirMashup.svelte';

  interface Props {
    mashupId: number | null;
    onver: (id: number | null) => void;
  }

  let { mashupId, onver }: Props = $props();
</script>

{#if mashupId !== null}
  <MashupActivo {mashupId} oncambio={onver} />
{/if}
<PedirMashup onempezado={onver} />
