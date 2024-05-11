/**
 * This example shows how to use react and jsx in a plug-in.
 * This is a slightly more complex example to demonstrate a common
 * pattern for these types of UI plug-ins. 
 * 
 * It's helpful to model the plug-in as a state machine. You only need two exports:
 * 
 * + 1. `setState` takes an `Action` which steps or mutates the state machine
 * + 2. `render` renders the current state
 * 
 * You can store all the application's state in a single global variable.
 * Make sure it can only be mutated through incoming `Action` messages.
 * Use JSX to build up your application view and render with `renderToString`.
 * 
 * With this programming model you can build pretty complex plug-ins, like a
 * working chat application.
 * 
 * The one downside of course is the in-memory state. However, you could just
 * simply offer your plug-in developers a host function to mutate (or persist)
 * and fetch it.
 * 
 * This can be tested out with the `shell` command in the extism CLI
 *
 * $ make compile-examples 
 * $ extism shell
 * 
 * > extism call examples/react.wasm render --wasi
 * <div style="background-color:lightblue"><p>Hello</p></div>
 * > extism call examples/react.wasm setState --input='{"type": "SET_SETTING", "payload": { "backgroundColor": "tomato" }}' --wasi
 * <div style="background-color:tomato"><p>Hello</p></div>
 * > extism call examples/react.wasm render --wasi
 * <div style="background-color:tomato"><p>Hello</p></div>
 */
import { renderToString } from 'react-dom/server';
import React from 'react'

interface Settings {
  backgroundColor: string;
}

// We can store all our application's state here
interface AppState {
  settings: Settings;
}

// We provide a number of "actions" that can be passed to setState to mutate the state machine
enum ActionType {
  SetSetting = "SET_SETTING"
}

interface Action {
  action: ActionType;
}

// We just have one action that can set properties in `settings`
interface SetSettingAction extends Action {
  action: ActionType.SetSetting;
  payload: Settings;
}

// Let's just make our application state a global. This should
// be okay as people can only mutate this through setState.
const APP_STATE = { settings: {} }

// Send an action to this export to step the state machine
// For convenience, this returns the newly rendered view but it could
// return a list of errors or other useful things
export function setState() {
  const action: Action = JSON.parse(Host.inputString())
  switch (action.action) {
    case ActionType.SetSetting:
      const setSettingAction = action as SetSettingAction;
      Object.assign(APP_STATE.settings, setSettingAction.payload)
      break;
    default:
      throw new Error(`Uknown action ${action}`)
  }

  Host.outputString(renderApp())
}


// Simply render the whole app
// Note: we could accept props here
export function render() {
  Host.outputString(renderApp())
}

function renderApp() {
  const view = (
    <div style={{ backgroundColor: APP_STATE.settings.backgroundColor || 'lightblue' }}>
      <p>Hello</p>
    </div>
  )

  // use react-dom to render the app component
  return renderToString(view)
}
