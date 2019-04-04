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

let make = _children => {
  ...component,
  initialState: () => {view: ViewList},
  didMount: self => {
    let watcherID =
      ReasonReact.Router.watchUrl(url =>
        switch (url.hash, self.state.view) {
        | ("", _) => self.send(ShowList)
        | (country, _) => self.send(ShowCountry(country))
        }
      );
    self.onUnmount(() => ReasonReact.Router.unwatchUrl(watcherID));
  },
  reducer: (action, _state) =>
    switch (action) {
    | ShowCountry(country) => ReasonReact.Update({view: ViewCountry(country)})
    | ShowList => ReasonReact.Update({view: ViewList})
    },
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
