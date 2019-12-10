type name = {
  common: string,
  official: string,
};
type borders = {
  name,
  cca3: string,
};
type country = {
  name,
  cca3: string,
  region: string,
  subregion: string,
  flag: string,
  capital: array(string),
  tld: array(string),
  borders: Js.Array.t(borders),
};

/* Create a GraphQL Query by using the graphql_ppx */
module GetCountry = [%graphql
  {|
  query get_country($name: String!) {
    country(name: $name) @bsRecord {
      name @bsRecord {
        common
        official
      }
      cca3
      region
      subregion
      flag
      capital
      tld
      borders @bsRecord {
        name @bsRecord {
          common
          official
        }
        cca3
      }
    }
  }
|}
];

module GetCountryQuery = ReasonApollo.CreateQuery(GetCountry);

let str = ReasonReact.string;
let component = ReasonReact.statelessComponent("Country");

let push = (path, event) => {
  ReactEvent.Mouse.preventDefault(event);
  ReasonReact.Router.push("#" ++ path);
};

let make = (~item: option(country), _children) => {
  ...component,
  render: self =>
    switch (item) {
    | None => <p> {"No such country found" |> str} </p>
    | Some(c) =>
      <div className="container">
        <div className="row">
          <div className="col-sm-3">
            <button className="primary" onClick={push("")}>
              {str("Back")}
            </button>
          </div>
          <div className="col-sm-9"> <h2> {str(c.name.common)} </h2> </div>
        </div>
        <div className="row">
          <div className="col-sm-3"> {str("Name")} </div>
          <div className="col-sm-9"> {str(c.name.common)} </div>
        </div>
        <div className="row">
          <div className="col-sm-3"> {str("Official name")} </div>
          <div className="col-sm-9"> {str(c.name.official)} </div>
        </div>
        <div className="row">
          <div className="col-sm-3"> {str("Capital")} </div>
          <div className="col-sm-9">
            {str(Js.Array.joinWith(",", c.capital))}
          </div>
        </div>
        <div className="row">
          <div className="col-sm-3"> {str("Region")} </div>
          <div className="col-sm-9"> {str(c.region)} </div>
        </div>
        <div className="row">
          <div className="col-sm-3"> {str("Subregion")} </div>
          <div className="col-sm-9"> {str(c.subregion)} </div>
        </div>
        <div className="row">
          <div className="col-sm-3"> {str("Flag")} </div>
          <div className="col-sm-9"> {str(c.flag)} </div>
        </div>
        <div className="row">
          <div className="col-sm-3"> {str("Borders")} </div>
          <div className="col-sm-9">
            <ul>
              {
                c.borders
                |> Array.map((item: borders) =>
                     <li key={item.cca3} onClick={push(item.cca3)}>
                       {item.name.common |> str}
                     </li>
                   )
                |> ReasonReact.array
              }
            </ul>
          </div>
        </div>
      </div>
    },
};
