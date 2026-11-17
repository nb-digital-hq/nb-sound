/** Los saludos de Inicio, en inglés. */

import type { TextosSaludos } from './es-saludos';
import type { Forma } from './forma';

export const enSaludos: Forma<TextosSaludos> = {
  madrugada: [
    (nombre: string) => `Good night, ${nombre}. Music keeps these quiet hours company.`,
    (nombre: string) => `A calm late night, ${nombre}. A good selection to enjoy unhurried.`,
    (nombre: string) => `Good night, ${nombre}. Your library is still awake with you.`,
    (nombre: string) => `${nombre}, listen calmly through this stretch of the night.`,
    (nombre: string) => `Good night, ${nombre}. The best time for attentive listening.`
  ],
  manana: [
    (nombre: string) => `Good morning, ${nombre}. Your music is ready.`,
    (nombre: string) => `Start the day with a good selection, ${nombre}.`,
    (nombre: string) => `Good morning, ${nombre}. Your library was waiting for you.`,
    (nombre: string) => `Today can start with something you already know, ${nombre}.`,
    (nombre: string) => `Good day, ${nombre}. There is music waiting to be picked up again.`,
    (nombre: string) => `${nombre}, start with a song that sets the pace.`,
    (nombre: string) => `Mornings sound better when you choose well, ${nombre}.`,
    (nombre: string) => `Your library has good material for today, ${nombre}.`,
    (nombre: string) => `${nombre}, go back to what you were listening to.`,
    (nombre: string) => `Good morning, ${nombre}. Pick the first tone of the day.`,
    (nombre: string) => `There is a session waiting for you, ${nombre}.`,
    (nombre: string) => `${nombre}, your favorites are ready to play again.`,
    (nombre: string) => `Good day, ${nombre}. Your library is in order.`,
    (nombre: string) => `${nombre}, open the morning with something of your own.`,
    (nombre: string) => `The first song can change the pace, ${nombre}.`,
    (nombre: string) => `Good morning, ${nombre}. See what sounds best right now.`
  ],
  tarde: [
    (nombre: string) => `Good afternoon, ${nombre}. Your next listen is close.`,
    (nombre: string) => `Music break, ${nombre}. There is plenty to pick up again.`,
    (nombre: string) => `Good afternoon, ${nombre}. Play something that moves you.`,
    (nombre: string) => `Your afternoon can carry on with good music, ${nombre}.`,
    (nombre: string) => `Keep the rhythm going, ${nombre}.`,
    (nombre: string) => `${nombre}, a good playlist brings order to any afternoon.`,
    (nombre: string) => `Your library has options for this hour, ${nombre}.`,
    (nombre: string) => `Perfect time to rediscover favorites, ${nombre}.`,
    (nombre: string) => `Good afternoon, ${nombre}. Make room for a different song.`,
    (nombre: string) => `${nombre}, this afternoon can sound more intentional.`,
    (nombre: string) => `${nombre}, go back to the songs you left halfway.`,
    (nombre: string) => `Your library has just what you need now, ${nombre}.`,
    (nombre: string) => `Good afternoon, ${nombre}. Some albums deserve another spin.`,
    (nombre: string) => `${nombre}, choose something familiar or try another path.`,
    (nombre: string) => `The afternoon is ready for a short session, ${nombre}.`,
    (nombre: string) => `${nombre}, your regular artists are waiting.`
  ],
  noche: [
    (nombre: string) => `Good evening, ${nombre}. Time to listen unhurried.`,
    (nombre: string) => `Good evening, ${nombre}. Your music can slow things down.`,
    (nombre: string) => `Close the day with the right song, ${nombre}.`,
    (nombre: string) => `Let something well chosen play tonight, ${nombre}.`,
    (nombre: string) => `Slow down, ${nombre}, and let your library play.`,
    (nombre: string) => `${nombre}, tonight deserves a fine selection.`,
    (nombre: string) => `Night mode, ${nombre}. Better with a calm selection.`,
    (nombre: string) => `Good evening, ${nombre}. The best end to the day starts with play.`,
    (nombre: string) => `${nombre}, let a good song do the rest.`,
    (nombre: string) => `This is your moment, ${nombre}: calm, rhythm and good music.`,
    (nombre: string) => `May the night find just the right song, ${nombre}.`,
    (nombre: string) => `${nombre}, put some music to this part of the day.`,
    (nombre: string) => `Good evening, ${nombre}. Some albums are best heard slowly.`,
    (nombre: string) => `${nombre}, go back to your trusted songs.`,
    (nombre: string) => `The night calls for more careful listening, ${nombre}.`,
    (nombre: string) => `Good evening, ${nombre}. Your library is still awake.`
  ],
  sinNombre: [
    'What do you want to listen to today?',
    'Play something to go with this moment.',
    'Your next favorite song can start right now.',
    'Today is a great day to discover new music.',
    'Your library is ready: you just have to choose.',
    'Make this moment sound better.',
    'Explore something new: a song can surprise you.',
    'Pick a rhythm and let everything fall into place.',
    'Ready for a session worth your time?',
    'Press play: good discoveries start here.',
    'Today’s soundtrack is waiting for you.',
    'A good song always arrives on time.',
    'Give that track you have not heard yet a chance.',
    'Try a different selection without leaving your library.',
    'Today can sound amazing too.',
    'Open the session: there is music for every moment.',
    'Go back to what you were listening to.',
    'Your favorites are ready to play again.',
    'Your library has something for this hour.',
    'Pick a song and let the session flow.',
    'Rediscover an album you have not played in a while.',
    'Some artists are waiting for another spin.',
    'Your local music has a lot to show.'
  ]
};
