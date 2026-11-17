/**
 * Los saludos de Inicio, en español. Van aparte del arranque: son muchos y solo los
 * usa el saludo, que puede esperar a que lleguen.
 */
export const esSaludos = {
  madrugada: [
    (nombre: string) => `Buenas noches, ${nombre}. La música acompaña la calma de estas horas.`,
    (nombre: string) =>
      `Madrugada tranquila, ${nombre}. Una buena selección para escuchar sin prisa.`,
    (nombre: string) => `Buenas noches, ${nombre}. Tu biblioteca sigue despierta contigo.`,
    (nombre: string) => `${nombre}, escucha con calma en este tramo de la noche.`,
    (nombre: string) => `Buenas noches, ${nombre}. El mejor momento para una escucha atenta.`
  ],
  manana: [
    (nombre: string) => `Buenos días, ${nombre}. Tu música ya está lista.`,
    (nombre: string) => `Arranca el día con una buena selección, ${nombre}.`,
    (nombre: string) => `Buenos días, ${nombre}. Tu biblioteca te estaba esperando.`,
    (nombre: string) => `Hoy puede empezar con algo que ya conoces, ${nombre}.`,
    (nombre: string) => `Buen día, ${nombre}. Hay música pendiente por retomar.`,
    (nombre: string) => `${nombre}, empieza por una canción que marque el ritmo.`,
    (nombre: string) => `La mañana suena mejor cuando eliges bien, ${nombre}.`,
    (nombre: string) => `Tu biblioteca tiene buen material para hoy, ${nombre}.`,
    (nombre: string) => `${nombre}, vuelve a lo que estabas escuchando.`,
    (nombre: string) => `Buenos días, ${nombre}. Elige el primer tono del día.`,
    (nombre: string) => `Hay una sesión esperando por ti, ${nombre}.`,
    (nombre: string) => `${nombre}, tienes favoritos listos para volver a sonar.`,
    (nombre: string) => `Buen día, ${nombre}. Tu biblioteca está en orden.`,
    (nombre: string) => `${nombre}, abre la mañana con algo tuyo.`,
    (nombre: string) => `La primera canción puede cambiar el ritmo, ${nombre}.`,
    (nombre: string) => `Buenos días, ${nombre}. Revisa qué suena mejor ahora.`
  ],
  tarde: [
    (nombre: string) => `Buenas tardes, ${nombre}. Tu próxima escucha está cerca.`,
    (nombre: string) => `Pausa musical, ${nombre}. Hay mucho por retomar.`,
    (nombre: string) => `Buenas tardes, ${nombre}. Dale play a algo que te mueva.`,
    (nombre: string) => `Tu tarde puede seguir con buena música, ${nombre}.`,
    (nombre: string) => `Que no decaiga el ritmo, ${nombre}.`,
    (nombre: string) => `${nombre}, una buena playlist ordena cualquier tarde.`,
    (nombre: string) => `Tu biblioteca tiene opciones para esta hora, ${nombre}.`,
    (nombre: string) => `Hora perfecta para redescubrir favoritos, ${nombre}.`,
    (nombre: string) => `Buenas tardes, ${nombre}. Dale espacio a una canción distinta.`,
    (nombre: string) => `${nombre}, esta tarde puede sonar con más intención.`,
    (nombre: string) => `${nombre}, vuelve a esas canciones que dejaste a medias.`,
    (nombre: string) => `Tu biblioteca tiene justo lo que necesitas ahora, ${nombre}.`,
    (nombre: string) => `Buenas tardes, ${nombre}. Hay álbumes que merecen otra vuelta.`,
    (nombre: string) => `${nombre}, elige algo familiar o prueba otra ruta.`,
    (nombre: string) => `La tarde está lista para una sesión corta, ${nombre}.`,
    (nombre: string) => `${nombre}, tus artistas frecuentes están esperando.`
  ],
  noche: [
    (nombre: string) => `Buenas noches, ${nombre}. Hora de escuchar sin prisa.`,
    (nombre: string) => `Buenas noches, ${nombre}. Tu música puede bajar el ritmo.`,
    (nombre: string) => `Cierra el día con la canción correcta, ${nombre}.`,
    (nombre: string) => `Que suene algo bien elegido esta noche, ${nombre}.`,
    (nombre: string) => `Baja el ritmo, ${nombre}, y deja que suene tu biblioteca.`,
    (nombre: string) => `${nombre}, esta noche merece una selección fina.`,
    (nombre: string) => `Modo nocturno, ${nombre}. Mejor con una selección tranquila.`,
    (nombre: string) => `Buenas noches, ${nombre}. El mejor cierre del día empieza con play.`,
    (nombre: string) => `${nombre}, deja que una buena canción haga el resto.`,
    (nombre: string) => `Es tu momento, ${nombre}: calma, ritmo y buena música.`,
    (nombre: string) => `Que la noche encuentre una canción precisa, ${nombre}.`,
    (nombre: string) => `${nombre}, ponle música a este tramo del día.`,
    (nombre: string) => `Buenas noches, ${nombre}. Hay álbumes para escuchar con calma.`,
    (nombre: string) => `${nombre}, vuelve a tus canciones de confianza.`,
    (nombre: string) => `La noche pide una escucha más cuidada, ${nombre}.`,
    (nombre: string) => `Buenas noches, ${nombre}. Tu biblioteca sigue despierta.`
  ],
  sinNombre: [
    '¿Qué quieres escuchar hoy?',
    'Dale play a algo que acompañe este momento.',
    'Tu próxima canción favorita puede empezar ahora.',
    'Hoy es un gran día para descubrir nueva música.',
    'Tu biblioteca está lista: solo falta que elijas.',
    'Haz que este momento suene mejor.',
    'Explora algo nuevo: una canción puede sorprenderte.',
    'Elige un ritmo y deja que todo encaje.',
    '¿Listo para una sesión que valga la pena?',
    'Pulsa play: aquí empiezan los buenos descubrimientos.',
    'Tu soundtrack de hoy te está esperando.',
    'Una buena canción siempre llega a tiempo.',
    'Dale una oportunidad a ese tema que aún no has escuchado.',
    'Prueba una selección distinta sin salir de tu biblioteca.',
    'Hoy también puede sonar increíble.',
    'Abre la sesión: hay música para cada momento.',
    'Vuelve a lo que estabas escuchando.',
    'Hay favoritos listos para sonar de nuevo.',
    'Tu biblioteca tiene algo para esta hora.',
    'Elige una canción y deja que la sesión avance.',
    'Redescubre un álbum que no escuchas hace rato.',
    'Hay artistas esperando otra vuelta.',
    'Tu música local tiene mucho por mostrar.'
  ]
};

export type TextosSaludos = typeof esSaludos;
