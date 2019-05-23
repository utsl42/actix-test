open CountriesList;
open Country;

let str = ReasonReact.string;

type view =
  | ViewCountry(string)
  | ViewList;

type state = {view};

type action =
  | ShowCountry(string)
  | ShowList;

let component = ReasonReact.reducerComponent("App");

/* Map a urlhash to an action. With this, we can convert browser navigation into reducer actions. */
let urlhash2action = (hash, view) =>
  switch (hash, view) {
  | ("", _) => ShowList /* default to list view */
  | (country, _) => ShowCountry(country)
  };

/* Map an action to a view we want to switch to. */
let action2view = action =>
  switch (action) {
  | ShowCountry(country) => ViewCountry(country)
  | ShowList => ViewList
  };

let make = _children => {
  ...component,
  initialState: () => {
    view:
      /* retrieve the initial view from the url hash */
      action2view(
        urlhash2action(
          ReasonReact.Router.dangerouslyGetInitialUrl().hash,
          ViewList,
        ),
      ),
  },
  didMount: self => {
    let watcherID =
      ReasonReact.Router.watchUrl(url
        /* update view on url change */
        => self.send(urlhash2action(url.hash, self.state.view)));
    self.onUnmount(() => ReasonReact.Router.unwatchUrl(watcherID));
  },
  reducer: (action, _state) =>
    ReasonReact.Update({view: action2view(action)}),
  render: self =>
    <div>
      <h1> {"ReasonReact + GraphQL Countries" |> str} </h1>
      {
        switch (self.state.view) {
        | ViewList =>
          let getCountriesListQuery = GetCountriesList.make();
          <GetCountriesQuery variables=getCountriesListQuery##variables>
            ...(
                 ({result}) =>
                   switch (result) {
                   | Loading => <div> {"Loading!" |> str} </div>
                   | Error(error) => <div> {error##message |> str} </div>
                   | Data(data) => <CountriesList items=data##listCountries />
                   }
               )
          </GetCountriesQuery>;
        | ViewCountry(c) =>
          let getCountryQuery = GetCountry.make(~name=c, ());
          <GetCountryQuery variables=getCountryQuery##variables>
            ...(
                 ({result}) =>
                   switch (result) {
                   | Loading => <div> {"Loading!" |> str} </div>
                   | Error(error) => <div> {error##message |> str} </div>
                   | Data(data) => <Country item=data##country />
                   }
               )
          </GetCountryQuery>;
        }
      }
    </div>,
};