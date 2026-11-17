<!--
  @component
  La marca de NB Sound: un solo dibujo que toma los colores del tema.

  La forma vive en `static/marca.svg` y se pinta como máscara: el disco es
  `--m3c-primary` y la silueta `--m3c-on-primary`, así que cambiar de tema cambia
  el color y nunca el dibujo. Se descartó un PNG por tema (63 archivos que había
  que regenerar con cada tema nuevo) y el SVG en línea (su trazado pesaría en el
  presupuesto del arranque).

  El tamaño lo da quien la usa con `--logo-tamano`.
-->
<span class="logo" aria-hidden="true"></span>

<style>
  .logo {
    position: relative;
    display: block;
    flex-shrink: 0;
    width: var(--logo-tamano, 4rem);
    height: var(--logo-tamano, 4rem);
    border-radius: var(--ns-radio-pleno);
    background: var(--m3c-primary);
    transition:
      width var(--ns-duracion-corta) var(--ns-curva),
      height var(--ns-duracion-corta) var(--ns-curva);
  }

  /* En el dibujo original el disco ocupa 968 de 1024 unidades. Aquí el disco es
     la caja entera, así que la máscara sobresale un 2,9 % por lado para que la
     silueta conserve su proporción respecto al disco. */
  .logo::before {
    position: absolute;
    inset: -2.9%;
    background: var(--m3c-on-primary);
    content: '';
    -webkit-mask: url('/marca.svg') center / 100% 100% no-repeat;
    mask: url('/marca.svg') center / 100% 100% no-repeat;
  }
</style>
