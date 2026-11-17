/**
 * La forma que debe tener un diccionario traducido.
 *
 * El español es la fuente y se escribe `as const`, así que su tipo trae cada
 * cadena como literal exacto. Un diccionario en otro idioma no puede repetir esos
 * literales (son otras cadenas), pero sí debe tener **las mismas claves, la misma
 * anidación y las mismas firmas**. `Forma` ensancha los literales a `string` y
 * conserva todo lo demás: falta una clave en inglés o en portugués y el proyecto no
 * compila. "Está todo traducido" pasa a ser una propiedad comprobada.
 */
export type Forma<T> = T extends string
  ? string
  : T extends (...args: infer A) => infer R
    ? (...args: A) => Forma<R>
    : T extends readonly (infer E)[]
      ? readonly Forma<E>[]
      : T extends object
        ? { -readonly [K in keyof T]: Forma<T[K]> }
        : T;
