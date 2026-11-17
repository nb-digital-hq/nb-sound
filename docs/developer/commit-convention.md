# Convención de commits

> **Estado:** Aplicable. La CI comprueba el asunto y la firma de origen de cada commit de una propuesta.

## Formato

```text
tipo(ámbito): resumen en minúsculas, en presente

Cuerpo: por qué se hace el cambio y qué se verificó. No una lista de archivos.

Signed-off-by: Nombre <correo>
```

Tipos: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci`, `chore`. El ámbito es el área
(`reproductor`, `catalogo`, `dj`, `movil`, `i18n`…).

## Declaración de origen

Cada commit lleva `Signed-off-by` (`git commit -s`): quien contribuye certifica el
[Developer Certificate of Origin](https://developercertificate.org/), es decir, que tiene derecho a aportar ese código
bajo la licencia del proyecto.

## Ramas y propuestas

`main` está protegida: los cambios entran por pull request, con la CI en verde. No se reescribe su historia.
