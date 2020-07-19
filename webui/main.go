package main

import (
	"encoding/json"
	"fmt"
	"io/ioutil"
	"log"
	"net/http"
)

type interactInputJSON struct {
	State string `json:"state"`
	X     int    `json:"x"`
	Y     int    `json:"y"`
}

type interactOutputJSON struct {
	State      string        `json:"state"`
	PointsList [][]pointJSON `json:"pointsList"`
}

type pointJSON struct {
	X int `json:"x"`
	Y int `json:"y"`
}

func interactHandler(w http.ResponseWriter, req *http.Request) {
	if req.Method != "POST" {
		w.WriteHeader(http.StatusBadRequest)
		fmt.Fprintln(w, "メソッドはPOSTで")
		return
	}

	reqBody, err := ioutil.ReadAll(req.Body)
	if err != nil {
		w.WriteHeader(http.StatusInternalServerError)
		fmt.Fprintln(w, "リクエストボディを読めなかった:", err)
		return
	}

	var input interactInputJSON
	if err := json.Unmarshal(reqBody, &input); err != nil {
		w.WriteHeader(http.StatusBadRequest)
		fmt.Fprintln(w, "JSONがパースできなかった:", err)
		return
	}

	// TODO: ここで処理する
	log.Printf("Request: x=%v, y=%v, state=%v", input.X, input.Y, input.State)

	output := interactOutputJSON{
		State: input.State,
		PointsList: [][]pointJSON{
			{
				{X: 0, Y: 0},
				{X: 0, Y: 1},
				{X: 1, Y: 0},
				{X: 1, Y: 1},
			},
		},
	}
	respBody, err := json.Marshal(&output)
	if err != nil {
		w.WriteHeader(http.StatusInternalServerError)
		fmt.Fprintln(w, "レスポンスの生成に失敗: ", err)
		return
	}
	w.Header().Set("Content-Type", "application/json")
	w.Write(respBody)
}

func main() {
	http.HandleFunc("/interact", interactHandler)

	log.Print("server start")
	log.Fatal(http.ListenAndServe(":8000", nil))
}
