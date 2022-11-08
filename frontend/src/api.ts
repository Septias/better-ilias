export class Api {
  ws: WebSocket
  constructor() {
    this.ws = new WebSocket('ws://localhost:2654')
    this.ws.addEventListener('open', event => console.log('hi'))
    this.ws.addEventListener('message', (event) => {
      console.log('Message from server ', event.data)
    })
  }

  update() {

  }

  login() {

  }

  add_update_listener(listener: (data: any) => void) {
    this.ws.addEventListener('message', (event) => {
      listener(event.data)
    })
  }
}
