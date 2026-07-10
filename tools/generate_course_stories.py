#!/usr/bin/env python3
"""Create static LexiPath story assets with an OpenAI-compatible endpoint."""
from __future__ import annotations

import argparse, json, os, re, sys, time
from pathlib import Path
from urllib.request import Request, urlopen

NAMES={"Alex","Anna","Ben","Emma","Jack","Leo","Lily","Lucy","Mia","Nina","Sam","Tom"}
CONNECTORS={"after","although","because","before","but","however","if","so","then","though","when","while"}
IRREG={"be":{"am","is","are","was","were","been","being"},"begin":{"began","begun","beginning","begins"},"buy":{"bought","buying","buys"},"do":{"did","done","doing","does"},"feel":{"felt","feeling","feels"},"find":{"found","finding","finds"},"give":{"gave","given","giving","gives"},"go":{"went","gone","going","goes"},"have":{"had","having","has"},"hear":{"heard","hearing","hears"},"leave":{"left","leaving","leaves"},"make":{"made","making","makes"},"read":{"reading","reads"},"say":{"said","saying","says"},"see":{"saw","seen","seeing","sees"},"sit":{"sat","sitting","sits"},"take":{"took","taken","taking","takes"},"tell":{"told","telling","tells"},"think":{"thought","thinking","thinks"},"write":{"wrote","written","writing","writes"}}
TOKEN_RE=re.compile(r"[A-Za-z]+(?:'[A-Za-z]+)?")


def tokens(text:str)->list[str]:
    out=[]
    for raw in TOKEN_RE.findall(text):
        low=raw.lower()
        out.append(low[:-2] if low.endswith("'s") else low)
    return out


def forms(word:str)->set[str]:
    w=word.lower(); out={w}|IRREG.get(w,set())
    if " " in w: return out
    if w.endswith("e"):
        out|={w+"s",w[:-1]+"ing",w+"d"}
    elif w.endswith("y") and len(w)>1 and w[-2] not in "aeiou":
        out|={w[:-1]+"ies",w[:-1]+"ied",w+"ing"}
    elif len(w)>2 and w[-1] not in "aeiouywx" and w[-2] in "aeiou" and w[-3] not in "aeiou":
        out|={w+"s",w+w[-1]+"ed",w+w[-1]+"ing"}
    else: out|={w+"s",w+"ed",w+"ing"}
    return out


def load(path:Path, default):
    return json.loads(path.read_text(encoding="utf-8")) if path.exists() else default


def lessons(course:dict):
    learned=[]
    for stage in course["stages"]:
        for lesson in stage["lessons"]:
            current=[w["text"] for w in lesson["new_words"]]
            learned.extend(current)
            yield stage,lesson,list(learned)


def level_of(stage:dict)->str:
    text=(stage.get("id","")+" "+stage.get("title","")).upper()
    return next((x for x in ("A1","A2","B1","B2") if x in text),"A1")


def profile(level:str):
    return {"A1":(10,16,16,4),"A2":(12,18,20,5),"B1":(14,22,24,6),"B2":(16,26,28,7)}[level]


def allowed_words(known:list[str], characters:list[str])->set[str]:
    out={n.lower() for n in characters}
    for entry in known:
        parts=tokens(entry)
        for part in parts: out|=forms(part)
    return out


def count_target(sentences:list[str], target:str)->tuple[int,int]:
    t=tokens(target)
    if len(t)!=1:
        joined=[tokens(s) for s in sentences]
        hits=sum(sum(1 for i in range(len(x)-len(t)+1) if x[i:i+len(t)]==t) for x in joined)
        return hits,sum(any(x[i:i+len(t)]==t for i in range(len(x)-len(t)+1)) for x in joined)
    variants=forms(t[0]); exact=0; separate=0
    for sentence in sentences:
        st=tokens(sentence); exact+=st.count(t[0]); separate+=int(any(x in variants for x in st))
    return exact,separate


def validate_story(story:dict, lesson:dict, known:list[str])->list[str]:
    issues=[]; field=story.get("lesson_id","?")
    level=story.get("level",""); chars=story.get("characters",[]); sents=story.get("sentences",[]); arc=story.get("arc",{})
    if story.get("lesson_id")!=lesson["id"]: issues.append(f"{field}: lesson id mismatch")
    if not story.get("title"): issues.append(f"{field}: title is empty")
    if level not in {"A1","A2","B1","B2"}: issues.append(f"{field}: invalid level")
    if not 1<=len(chars)<=4 or any(x not in NAMES for x in chars) or len(set(chars))!=len(chars): issues.append(f"{field}: invalid characters")
    if level in {"A1","A2","B1","B2"}:
        lo,hi,max_words,min_conn=profile(level)
        if not lo<=len(sents)<=hi: issues.append(f"{field}: expected {lo}-{hi} sentences")
        if any(len(tokens(x))>max_words for x in sents): issues.append(f"{field}: sentence is too long")
        if len(CONNECTORS & set(tokens(" ".join(sents))))<min_conn: issues.append(f"{field}: too few connectors")
    if len({x.strip().lower() for x in sents})!=len(sents): issues.append(f"{field}: duplicate sentence")
    openings=[tokens(x)[:2] for x in sents]
    if any(openings[i]==openings[i+1]==openings[i+2] for i in range(max(0,len(openings)-2))): issues.append(f"{field}: repeated opening frame")
    required=["setup_sentence","goal_sentence","problem_sentence","attempt_sentences","turn_sentence","resolution_sentence"]
    if any(k not in arc for k in required): issues.append(f"{field}: incomplete narrative arc")
    else:
        idx=[arc["setup_sentence"],arc["goal_sentence"],arc["problem_sentence"],arc["turn_sentence"],arc["resolution_sentence"]]
        extra=arc["attempt_sentences"]+([arc["reveal_sentence"]] if arc.get("reveal_sentence") is not None else [])
        if any(not isinstance(i,int) or i<0 or i>=len(sents) for i in idx+extra): issues.append(f"{field}: arc index out of range")
        elif not(idx[0]<=idx[1]<idx[2]<idx[3]<idx[4]) or len(arc["attempt_sentences"])<2 or any(not idx[2]<i<idx[3] for i in arc["attempt_sentences"]): issues.append(f"{field}: invalid narrative order")
    allowed=allowed_words(known,chars)
    unknown=sorted(set(tokens(" ".join(sents)))-allowed)
    if unknown: issues.append(f"{field}: unknown words: {', '.join(unknown)}")
    for name in chars:
        if tokens(" ".join(sents)).count(name.lower())<2: issues.append(f"{field}: character {name} appears fewer than twice")
    for word in lesson["new_words"]:
        exact,separate=count_target(sents,word["text"])
        if exact<2: issues.append(f"{field}: target {word['text']} needs exact use in at least two story sentences")
    return issues


def prompt(stage:dict, lesson:dict, known:list[str])->str:
    level=level_of(stage); lo,hi,max_words,min_conn=profile(level)
    targets=[{"word":w["text"],"meaning":w["meaning"]} for w in lesson["new_words"]]
    contract={"lesson_id":lesson["id"],"level":level,"known_words":known,"targets":targets,"allowed_names":sorted(NAMES),"sentence_count":[lo,hi],"max_words_per_sentence":max_words,"minimum_distinct_connectors":min_conn}
    return """Write one coherent English micro-story for an English learner. Return JSON only. Use only known_words, ordinary noun/verb inflections, and declared allowed_names. Every target must appear in exact dictionary form in at least two different sentences. The story must have a setup, goal, concrete problem, at least two attempts, a turn, optional reveal, and a resolution that recalls an earlier object, action, or advice. Vary openings; do not create a list of example sentences. JSON schema: {lesson_id,title,level,characters,sentences,arc:{setup_sentence,goal_sentence,problem_sentence,attempt_sentences,turn_sentence,reveal_sentence,resolution_sentence}}. Sentence indexes are zero-based.\nCONTRACT:\n"""+json.dumps(contract,ensure_ascii=False,separators=(",",":"))


def call(endpoint:str,model:str,token:str,text:str)->dict:
    body=json.dumps({"model":model,"temperature":0.7,"response_format":{"type":"json_object"},"messages":[{"role":"system","content":"You write controlled-vocabulary narrative course content."},{"role":"user","content":text}]}).encode()
    req=Request(endpoint,data=body,headers={"Authorization":f"Bearer {token}","Content-Type":"application/json","Accept":"application/json"})
    with urlopen(req,timeout=180) as response: data=json.load(response)
    content=data["choices"][0]["message"]["content"]
    return json.loads(content)


def select(course:dict,args):
    chosen=[]
    for stage,lesson,known in lessons(course):
        if lesson["id"].startswith("stage-final-"): continue
        if args.lesson and lesson["id"]!=args.lesson: continue
        if args.stage and stage["id"]!=args.stage: continue
        chosen.append((stage,lesson,known))
    if not(args.lesson or args.stage or args.all or args.validate_only): raise SystemExit("choose --lesson, --stage, --all, or --validate-only")
    return chosen


def write(path:Path, stories:list[dict]):
    path.parent.mkdir(parents=True,exist_ok=True); tmp=path.with_suffix(path.suffix+".tmp")
    tmp.write_text(json.dumps(stories,ensure_ascii=False,indent=2)+"\n",encoding="utf-8"); tmp.replace(path)


def main()->int:
    p=argparse.ArgumentParser(description=__doc__); p.add_argument("course",type=Path); p.add_argument("--output",type=Path,default=Path("assets/course-stories/curated.json")); p.add_argument("--lesson"); p.add_argument("--stage"); p.add_argument("--all",action="store_true"); p.add_argument("--resume",action="store_true"); p.add_argument("--validate-only",action="store_true"); p.add_argument("--dry-run",action="store_true"); p.add_argument("--endpoint",default="https://models.github.ai/inference/chat/completions"); p.add_argument("--model",default="openai/gpt-4.1"); p.add_argument("--token-env",default="GITHUB_TOKEN"); p.add_argument("--max-retries",type=int,default=3); p.add_argument("--pause-seconds",type=float,default=1.0); args=p.parse_args()
    course=load(args.course,None); bank=load(args.output,[]); by_id={x["lesson_id"]:x for x in bank}; selected=select(course,args); all_lessons={x[1]["id"]:x for x in lessons(course)}
    if args.validate_only:
        issues=[]
        for story in bank:
            item=all_lessons.get(story.get("lesson_id")); issues.extend([f"{story.get('lesson_id')}: lesson not found"] if not item else validate_story(story,item[1],item[2]))
        if issues: print("\n".join(issues),file=sys.stderr); return 1
        print(f"Validated {len(bank)} curated stories."); return 0
    if args.dry_run:
        for stage,lesson,known in selected: print(prompt(stage,lesson,known)+"\n")
        return 0
    token=os.getenv(args.token_env)
    if not token: raise SystemExit(f"environment variable {args.token_env} is required")
    for stage,lesson,known in selected:
        if args.resume and lesson["id"] in by_id: continue
        text=prompt(stage,lesson,known); last=[]
        for attempt in range(1,args.max_retries+1):
            story=call(args.endpoint,args.model,token,text+("\nPrevious validation errors: "+" | ".join(last) if last else "")); last=validate_story(story,lesson,known)
            if not last: break
        else: raise SystemExit(f"{lesson['id']}: "+" | ".join(last))
        by_id[lesson["id"]]=story; write(args.output,[by_id[k] for k in sorted(by_id)]); print(f"accepted {lesson['id']}"); time.sleep(args.pause_seconds)
    return 0

if __name__=="__main__": raise SystemExit(main())
