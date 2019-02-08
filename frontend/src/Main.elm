module Main exposing (Actions, DatabaseState(..), Model, Msg(..), Page(..), WorkDay, WorkSheet, actionsDecoder, filesDecoder, init, main, subscriptions, update, uploadView, view, viewAction, viewActions, viewDay, viewDuration, viewWorkSheet, workDayDecoder, workSheetDecoder)

import Browser
import Browser.Navigation exposing (Key)
import Dict exposing (Dict)
import File exposing (File)
import Html exposing (..)
import Html.Attributes exposing (..)
import Html.Events exposing (..)
import Http
import Json.Decode as D
import Url exposing (Url)
import Url.Parser as Parser exposing ((</>), Parser, oneOf, s)



-- MAIN


main =
    Browser.application
        { init = init
        , onUrlChange = OnUrlChange
        , onUrlRequest = OnUrlRequest
        , view = view
        , update = update
        , subscriptions = subscriptions
        }



-- MODEL


type alias Actions =
    Dict String Int


type alias WorkDay =
    Dict String Actions


type alias WorkSheet =
    Dict String WorkDay


type alias Model =
    { page : Page
    , key : Key
    }


type Page
    = UploadingDatabase DatabaseState
    | Day String
    | AvailableDays
    | PageNone


type Route
    = Upload
    | ViewDay String
    | SelectAvailableDay
    | NotFound


type DatabaseState
    = Waiting
    | Uploading Float
    | Done WorkSheet
    | Fail


parseUrl : Url -> Route
parseUrl url =
    case Parser.parse routeParser url of
        Just route ->
            route

        Nothing ->
            NotFound


routeParser : Parser (Route -> a) a
routeParser =
    oneOf
        [ Parser.map ViewDay (s "dag" </> Parser.string)
        , Parser.map Upload (s "upload-registratie")
        , Parser.map SelectAvailableDay (s "kies-een-dag")
        ]


getPath : Route -> String
getPath route =
    case route of
        NotFound ->
            "/404"

        Upload ->
            "/upload-registratie"

        SelectAvailableDay ->
            "/kies-een-dag"

        ViewDay day ->
            "/dag/" ++ day


getPage : Route -> Page
getPage route =
    case route of
        NotFound ->
            PageNone

        Upload ->
            UploadingDatabase Waiting

        SelectAvailableDay ->
            AvailableDays

        ViewDay day ->
            Day day


init : () -> Url -> Key -> ( Model, Cmd Msg )
init _ url key =
    let
        page =
            parseUrl url |> getPage
    in
    ( { page = page, key = key }
    , Cmd.none
    )


workSheetDecoder : D.Decoder WorkSheet
workSheetDecoder =
    D.dict workDayDecoder


workDayDecoder : D.Decoder WorkDay
workDayDecoder =
    D.dict actionsDecoder


actionsDecoder : D.Decoder Actions
actionsDecoder =
    D.dict D.int



-- UPDATE


type Msg
    = GotFiles (List File)
    | GotProgress Http.Progress
    | Uploaded (Result Http.Error WorkSheet)
    | OnUrlRequest Browser.UrlRequest
    | OnUrlChange Url
    | Noop


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        GotFiles files ->
            ( { model | page = UploadingDatabase (Uploading 0) }
            , Http.request
                { method = "POST"
                , url = "http://localhost:3010/upload"
                , headers = []
                , body = Http.multipartBody (List.map (Http.filePart "file") files)
                , expect = Http.expectJson Uploaded workSheetDecoder
                , timeout = Nothing
                , tracker = Just "upload"
                }
            )

        GotProgress progress ->
            case progress of
                Http.Sending p ->
                    ( { model | page = UploadingDatabase (Uploading (Http.fractionSent p)) }, Cmd.none )

                Http.Receiving _ ->
                    ( model, Cmd.none )

        Uploaded result ->
            case result of
                Ok response ->
                    ( { model | page = UploadingDatabase (Done response) }, Cmd.none )

                Err _ ->
                    ( { model | page = UploadingDatabase Fail }, Cmd.none )

        OnUrlChange url ->
            let
                newPage =
                    parseUrl url |> getPage
            in
            ( { model | page = newPage }, Cmd.none )

        Noop ->
            ( model, Cmd.none )

        OnUrlRequest urlRequest ->
            case urlRequest of
                Browser.Internal url ->
                    ( model
                    , Browser.Navigation.pushUrl model.key (Url.toString url)
                    )

                Browser.External url ->
                    ( model
                    , Browser.Navigation.load url
                    )



-- SUBSCRIPTIONS


subscriptions : Model -> Sub Msg
subscriptions model =
    Http.track "upload" GotProgress



-- VIEW


viewNavBar : Html Msg
viewNavBar =
    Html.nav [ class "navbar navbar-expand-lg navbar-dark bg-dark" ]
        [ Html.a [ class "navbar-brand" ] [ text "Humako uren registratie" ]
        , div [ class "collapse navbar-collapse show" ]
            [ Html.ul [ class "navbar-nav" ]
                [ Html.li
                    [ class "nav-item" ]
                    [ Html.a [ class "nav-link", href (getPath Upload) ] [ text "Upload nieuwe registratie" ] ]

                --                , Html.li
                --                    [ class "nav-item" ]
                --                    [ Html.a [ class "nav-link", href (getPath (ViewDay "2019-01-01")) ] [ text "Gister" ] ]
                --                , Html.li
                --                    [ class "nav-item" ]
                --                    [ Html.a [ class "nav-link", href (getPath (ViewDay "2019-01-02")) ] [ text "Vandaag" ] ]
                , Html.li
                    [ class "nav-item" ]
                    [ Html.a [ class "nav-link", href (getPath SelectAvailableDay) ] [ text "Selecteer dag" ] ]
                ]
            ]
        ]


view : Model -> Browser.Document Msg
view model =
    let
        header =
            [ viewNavBar ]
    in
    case model.page of
        UploadingDatabase database ->
            { title = "Upload uren registratie", body = header ++ [ uploadView database ] }

        Day day ->
            { title = "Uren voor " ++ day, body = header ++ [ div [] [ text day ] ] }

        AvailableDays ->
            { title = "Kies een dag", body = [ div [] [ text "pick a day" ] ] }

        PageNone ->
            { title = "404", body = [ div [] [ text "Pagina niet gevonden" ] ] }


uploadView : DatabaseState -> Html Msg
uploadView database =
    case database of
        Waiting ->
            input
                [ type_ "file"
                , multiple True
                , on "change" (D.map GotFiles filesDecoder)
                ]
                []

        Uploading fraction ->
            h1 [] [ text (String.fromInt (round (100 * fraction)) ++ "%") ]

        Done worksheet ->
            Html.div [] [ h1 [] [ text "DONE" ], viewWorkSheet worksheet ]

        Fail ->
            h1 [] [ text "FAILED IMPORTING DATABASE" ]


viewDuration : Int -> String
viewDuration duration =
    let
        minutes =
            remainderBy 60 duration |> String.fromInt

        formattedMinutes =
            if String.length minutes == 1 then
                "0" ++ minutes

            else
                minutes

        hours =
            floor (toFloat duration / 60) |> String.fromInt
    in
    hours ++ ":" ++ formattedMinutes


viewAction : ( String, Int ) -> Html msg
viewAction ( action, duration ) =
    Html.tr [ class "action-row" ] [ Html.td [ class "action-title" ] [ text action ], Html.td [ class "duration" ] [ text (viewDuration duration) ] ]


viewActions : ( String, Actions ) -> Html msg
viewActions ( employee, actions ) =
    Html.tr []
        [ Html.td [ class "employee-name" ] [ text employee ]
        , Html.td []
            [ Html.table [ class "action" ]
                (Dict.toList actions
                    |> List.map viewAction
                )
            ]
        ]


viewDay : ( String, WorkDay ) -> Html msg
viewDay ( day, workDay ) =
    Html.tr [ class "day-row" ]
        [ Html.td [ class "day-title" ] [ text day ]
        , Html.td []
            [ Html.table [ class "employee" ]
                (Dict.toList workDay
                    |> List.map viewActions
                )
            ]
        ]


viewWorkSheet : WorkSheet -> Html msg
viewWorkSheet worksheet =
    Html.table [ class "work-sheet" ]
        (Dict.toList worksheet
            |> List.map viewDay
        )


filesDecoder : D.Decoder (List File)
filesDecoder =
    D.at [ "target", "files" ] (D.list File.decoder)
