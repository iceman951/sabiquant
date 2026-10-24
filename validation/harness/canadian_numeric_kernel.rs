// FR-CAD-33..60: actual private Canadian numeric kernel replay of every frozen N row.
//
// Appended by validation/generator/canadian_numeric_kernel_check.py to a
// hash-attested scratch copy of src/bonds/canadian/numeric.rs only. The runner
// transports immutable fixture operands/expectations to plain TSV beside the
// scratch Cargo.toml; no expected value is ever used as an input.

#[cfg(test)]
mod frozen_numeric_replay {
    use super::*;
    use super::super::{CanadianFrequency, CanadianPeriodKind, CanadianSourcePolicy,
        CanadianYieldSelection, Coordinate, Source};
    use crate::amounts::DecimalAmount;
    use crate::calendar::EndOfMonth;
    use crate::error::{NonFinite, ValueError};
    use crate::time::Date;
    use std::collections::BTreeMap;

    const DATA: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/primitive-inputs.tsv"));
    const QUANTITIES: [&str;6] = ["price derivative","price second derivative",
        "time weighted amount","modified duration","macaulay duration","convexity"];

    pub(super) fn number<T: std::str::FromStr>(value: &str) -> T where T::Err: fmt::Debug {
        value.parse().expect("strict transported integer")
    }
    pub(super) fn bits(value: &str) -> u64 { u64::from_str_radix(value,16).expect("transported IEEE bits") }
    pub(super) fn float(value: &str) -> f64 { f64::from_bits(bits(value)) }
    pub(super) fn boolean(value: &str) -> bool { number(value) }
    pub(super) fn magnitude(value: &str, out: &mut [u64]) -> bool {
        wide::clear(out);
        let negative = value.starts_with('-');
        let digits = value.strip_prefix('-').unwrap_or(value);
        for (index, part) in digits.as_bytes().rchunks(16).enumerate() {
            let value = u64::from_str_radix(std::str::from_utf8(part).expect("ASCII hex"),16).expect("hex limb");
            if index >= out.len() { assert_eq!(value,0,"transport width"); }
            else { out[index]=value; }
        }
        negative && !wide::is_zero(out)
    }
    pub(super) fn assert_magnitude(actual: &[u64], want: &str, label: &str) {
        let mut expected = [0_u64;243];
        let negative = magnitude(want,&mut expected);
        assert!(!negative,"{label}");
        assert_eq!(wide::cmp(actual,&expected),Ordering::Equal,"{label}");
    }
    fn frequency(f: u32) -> CanadianFrequency {
        match f { 2=>CanadianFrequency::Semiannual,4=>CanadianFrequency::Quarterly,
            12=>CanadianFrequency::Monthly,_=>panic!("bad transported frequency") }
    }
    fn metadata(convention: &str, f: u32, days: u32) -> Metadata {
        let day = Date::from_ymd(2025,1,1).expect("fixed valid day");
        let convention = if convention=="compound" { CanadianYieldConvention::Compound }
            else { CanadianYieldConvention::MoneyMarket };
        Metadata { source_policy:CanadianSourcePolicy::CanonicalIiac13LocalV1,
            accrual_start:day,original_issue_date:day,stated_maturity_date:day,
            true_maturity_date:day,first_coupon_date:day,last_coupon_start_date:day,
            first_period:CanadianPeriodKind::Regular,last_period:CanadianPeriodKind::Regular,
            frequency:frequency(f),coupon_per_100:DecimalAmount::new(0,0).expect("zero"),
            end_of_month:EndOfMonth::Ignore,direct_cycle_anchor:day,settlement_date:day,
            yield_selection:CanadianYieldSelection::Explicit(convention),yield_convention:convention,
            valuation_accrued_per_100:0.0,settlement_accrued_per_100:0.0,
            quote_accrued_per_100:0.0,days_to_true_maturity:days }
    }
    fn source(n: &str,d: &str) -> Source {
        let mut numerator=[0;5];let mut denominator=[0;5];
        assert!(!magnitude(n,&mut numerator));assert!(!magnitude(d,&mut denominator));
        Source {numerator,denominator}
    }
    fn bundle(lambda_num:u64,lambda_den:u32,days:u32,remaining:u32) -> Bundle {
        Bundle {index:17,amount:Source::ZERO,
            lambda:Coordinate{numerator:lambda_num,denominator:lambda_den},
            days_to_payment:days-remaining,days_payment_to_true:remaining}
    }
    pub(super) fn check_value(result:Result<f64,CanadianError>,want:&str,class:&str,
                   phase:&str,quantity:&str,index:Option<usize>,label:&str) {
        match class {
            "normal"|"subnormal"|"zero" => assert_eq!(result.expect(label).to_bits(),bits(want),"{label}"),
            "underflow" => match result.expect_err(label) {
                CanadianError::Value{index:actual,phase:actual_phase,cause:ValueError::ResultOutOfDomain{quantity:actual_quantity,value,..},..}=>{
                    assert_eq!(actual,index,"{label}");assert_eq!(actual_phase,phase,"{label}");
                    assert_eq!(actual_quantity,quantity,"{label}");assert_eq!(value.to_bits(),bits(want),"{label}");
                },other=>panic!("{label}: wrong underflow {other:?}")
            },
            "overflow" => match result.expect_err(label) {
                CanadianError::Value{index:actual,phase:actual_phase,cause:ValueError::ResultNotFinite{quantity:actual_quantity,value},..}=>{
                    assert_eq!(actual,index,"{label}");assert_eq!(actual_phase,phase,"{label}");
                    assert_eq!(actual_quantity,quantity,"{label}");
                    assert_eq!(value,if bits(want)&SIGN==0{NonFinite::PositiveInfinity}else{NonFinite::NegativeInfinity},"{label}");
                },other=>panic!("{label}: wrong overflow {other:?}")
            },_=>panic!("bad expected class {class}")
        }
    }
    pub(super) fn check_price(result:Result<CanadianPrice,CanadianError>,want:&str,class:&str,label:&str) {
        assert!(matches!(class,"normal"|"subnormal"|"zero"));
        assert_eq!(result.expect(label).value().to_bits(),bits(want),"{label}");
    }

    #[test]
    fn all_frozen_primitive_rows_bind_actual_functions() {
        let mut counts=BTreeMap::new();
        let mut native_classes=BTreeMap::new();
        for line in DATA.lines() {
            let c:Vec<_>=line.split('\t').collect();
            let op=c[0];let id=c[1];*counts.entry(op).or_insert(0_usize)+=1;
            match op {
                "log_argument"=>{
                    let f=number(c[2]);let y=float(c[3]);
                    if c[4]=="domain" {
                        let mut ws=Workspace::new();
                        assert!(matches!(ws.quote(metadata("compound",f,1),y),Err(CanadianError::Value{index:None,phase:"yield quotation",..})),"{id}");
                    } else {
                        let (branch,argument)=logarithm_argument(y,f);
                        let name=match branch {LogarithmBranch::Zero=>"zero",LogarithmBranch::Tiny=>"tiny",LogarithmBranch::Natural=>"ln",LogarithmBranch::NaturalOnePlus=>"ln1p"};
                        assert_eq!(name,c[4],"{id}");assert_eq!(argument.to_bits(),bits(c[5]),"{id}");
                    }
                },
                "log_scale"=>assert_eq!(logarithm_scale(float(c[3]),number(c[2])).to_bits(),bits(c[4]),"{id}"),
                "exponent"|"tiny_bound"=>{
                    let mut ws=Workspace::new();
                    let z=ws.logarithmic_exponent(float(c[3]),number(c[2]),Coordinate{numerator:number(c[4]),denominator:number(c[5])},17).expect(id);
                    assert_eq!(z.to_bits(),bits(c[6]),"{id}");
                },
                "guard"=>{
                    let z=float(c[4]);let action=requires_exponential(float(c[3]),z,number(c[2]),17);
                    if boolean(c[5]) {
                        match action.expect_err(id) {
                            CanadianError::Rate{index:Some(17),phase:"discount factor",cause:RateError::NumericalBoundary{quantity:"discount factor",logarithm},..}=>assert_eq!(logarithm.to_bits(),z.to_bits(),"{id}"),
                            other=>panic!("{id}: wrong boundary {other:?}")
                        }
                    } else {assert_eq!(u8::from(action.expect(id)),number::<u8>(c[6]),"{id}");}
                },
                "guard_box"=>{
                    let(mut lo,mut hi,mut bl,mut bh)=([0;20],[0;20],[0;20],[0;20]);
                    let ln=magnitude(c[2],&mut lo);let hn=magnitude(c[3],&mut hi);
                    let bn=magnitude(c[4],&mut bl);let bhn=magnitude(c[5],&mut bh);
                    let intersects=reaches_upper_endpoint(signed_cmp(&lo,ln,&bh,bhn))
                        &&reaches_lower_endpoint(signed_cmp(&hi,hn,&bl,bn));
                    assert_eq!(intersects,boolean(c[6]),"{id}");
                },
                "elementary"=>{
                    let x=float(c[3]);let observed=match c[2]{"ln"=>x.ln(),"ln1p"=>x.ln_1p(),"exp"=>x.exp(),_=>panic!("function")};
                    let kind=if observed.is_nan(){"nan"}else if observed.is_infinite(){"inf"}else if observed.to_bits()&!SIGN==0{"zero"}else{"finite_nonzero"};
                    *native_classes.entry(kind).or_insert(0_usize)+=1;
                    if c[2]=="exp" && (!observed.is_finite()||observed<=0.0) {
                        assert!(validation::result_within("discount factor",observed,Interval::greater_than(0.0)).is_err(),"{id}");
                    } else {
                        let mut a=[0;112];let mut lo=[0;112];let mut hi=[0;112];
                        let ln=magnitude(c[4],&mut lo);let hn=magnitude(c[5],&mut hi);
                        let ex:i32=number(c[6]);let(sig,e)=decode(observed);let common=e.min(ex);
                        assert!(wide::set_u128(&mut a,u128::from(sig)));
                        assert!(wide::shl_assign(&mut a,u32::try_from(e-common).expect("shift")));
                        assert!(wide::shl_assign(&mut lo,u32::try_from(ex-common).expect("shift")));
                        assert!(wide::shl_assign(&mut hi,u32::try_from(ex-common).expect("shift")));
                        assert!(signed_cmp(&a,observed.is_sign_negative(),&lo,ln)!=Ordering::Less,"{id}: native below allowed");
                        assert!(signed_cmp(&a,observed.is_sign_negative(),&hi,hn)!=Ordering::Greater,"{id}: native above allowed");
                    }
                },
                "mm_affine"=>{
                    let y=float(c[2]);let days=number(c[3]);let rem=number(c[4]);
                    let mut ws=Workspace::new();let q=ws.quote(metadata("mm",2,days),y);
                    if boolean(c[5]) {
                        let y=q.expect(id);let b=bundle(1,1,days,rem);ws.mm_numerator(y,&b).expect(id);
                        assert_magnitude(&ws.d,c[6],id);assert_magnitude(&ws.n,c[7],id);
                        let factor=ws.factor(metadata("mm",2,days),y,0.0,&b,&mut |_,_|panic!("MM cannot call exp")).expect(id);
                        assert_eq!(factor.to_bits(),bits(c[8]),"{id}");
                    } else {assert!(matches!(q,Err(CanadianError::MoneyMarketDomain{..})),"{id}");}
                },
                "source_ieee"|"pv"=>{
                    let s=source(c[2],c[3]);
                    if op=="source_ieee" {assert_eq!(s.observe(Some(17),"source payable coupon").expect(id).to_bits(),bits(c[5]),"{id}");}
                    else {let mut ws=Workspace::new();let mut b=bundle(1,1,1,0);b.amount=s;
                        check_value(ws.present_value(float(c[4]),&b),c[5],c[6],"source present value","canadian present value",Some(17),id);}
                },
                "extended"=>{
                    let(mut n,mut d,mut u,mut v,mut aligned)=([0;112],[0;112],[0;112],[0;112],[0;112]);
                    let negative=magnitude(c[2],&mut n);assert!(!magnitude(c[3],&mut d));
                    let(q,e)=wide::round_to_extended53(&n,&d,number(c[4]),&mut u,&mut v,&mut aligned).expect(id);
                    assert_eq!((if q==0{0}else if negative{-1}else{1},q,e),(number::<i32>(c[5]),number::<u64>(c[6]),number::<i32>(c[7])),"{id}");
                },
                "risk_terms"=>{
                    let convention=c[2];let f=number(c[3]);let days=number(c[8]);let rem=number(c[9]);
                    let meta=metadata(convention,f,days);let mut ws=Workspace::new();let y=ws.quote(meta,float(c[4])).expect(id);
                    let b=bundle(number(c[6]),number(c[7]),days,rem);
                    if convention=="mm" {ws.mm_numerator(y,&b).expect(id);}
                    let mut sums=[[0;243];3];ws.risk_terms(meta,y,&b,float(c[5]),&mut sums).expect(id);
                    for k in 0..3 {let expected=c[10+k];let magnitude=expected.strip_prefix('-').unwrap_or(expected);assert_magnitude(&sums[k],magnitude,id);}
                },
                "retained"=>{
                    let mut sums=[[0;243];3];let mut negative=[false;3];
                    for k in 0..3 {negative[k]=magnitude(c[3+k],&mut sums[k]);}
                    for k in 0..6 {let which=[0,1,2,0,2,1][k];let sign=if k==3{!negative[which]&&!wide::is_zero(&sums[which])}else{negative[which]};
                        let denominator=if k<3{None}else{Some(float(c[2]))};
                        check_value(risk_value(&sums[which],sign,denominator,QUANTITIES[k],0.0),c[6+2*k],c[7+2*k],QUANTITIES[k],QUANTITIES[k],None,id);
                    }
                },
                "dirty_sum"=>{
                    let values:Vec<_>=c[2].split(',').map(float).collect();let got=dirty_price(&values);
                    if c[4]=="overflow" {match got.expect_err(id){CanadianError::Sum{phase:"dirty price reduction",cause:crate::math::SumError::Arithmetic{index:None,error:ValueError::ResultNotFinite{quantity:"exact sum",value}},..}=>assert_eq!(value,NonFinite::PositiveInfinity,"{id}"),other=>panic!("{id}: {other:?}")}}
                    else {assert_eq!(got.expect(id).to_bits(),bits(c[3]),"{id}");}
                },
                "price_bases"=>{
                    let mut meta=metadata(c[2],2,1);meta.valuation_accrued_per_100=float(c[4]);meta.settlement_accrued_per_100=float(c[5]);
                    meta.quote_accrued_per_100=if c[2]=="compound"{float(c[4])}else{float(c[5])};
                    for (k,basis) in [CanadianPriceBasis::ModelDirty,CanadianPriceBasis::Clean,CanadianPriceBasis::SettlementDirty].into_iter().enumerate(){
                        check_price(bridge(meta,float(c[3]),basis),c[6+2*k],c[7+2*k],id);
                    }
                },
                "target"=>{
                    let mut meta=metadata(c[2],2,1);meta.quote_accrued_per_100=float(c[5]);meta.settlement_accrued_per_100=float(c[6]);
                    let basis=match c[3]{"Clean"=>CanadianPriceBasis::Clean,"ModelDirty"=>CanadianPriceBasis::ModelDirty,"SettlementDirty"=>CanadianPriceBasis::SettlementDirty,_=>panic!("basis")};
                    let target=CanadianPrice::new(float(c[4]),basis).expect(id);let resolved=resolve_target(meta,target).expect(id);
                    assert_eq!(resolved.to_bits(),bits(c[7]),"{id}");assert_eq!(resolved>0.0,boolean(c[9]),"{id}");
                },
                "residual"=>{
                    let dirty=float(c[2]);let target=float(c[3]);let scale=number(c[4]);
                    assert_eq!(residual(dirty,target,0).expect(id).to_bits(),bits(c[5]),"{id}");
                    assert_eq!(residual(dirty,target,scale).expect(id).to_bits(),bits(c[7]),"{id}");
                },
                _=>panic!("unbound operation {op}")
            }
        }
        assert_eq!(counts.values().sum::<usize>(),2327);
        assert_eq!(counts.len(),17);
        println!("ACTUAL_NUMERIC_PRIMITIVE_ROWS {:?}; NATIVE_CURRENT_HOST {:?}",counts,native_classes);
    }
}

#[cfg(test)]
mod frozen_dated_replay {
    use super::*;
    use super::super::{CanadianBond,CanadianCouponSchedule,CanadianFrequency,
        CanadianSourcePolicy,CanadianYieldSelection,Geometry};
    use super::frozen_numeric_replay::{number,float,bits,assert_magnitude,
        check_value,check_price};
    use crate::amounts::DecimalAmount;
    use crate::calendar::EndOfMonth;
    use crate::time::Date;
    use std::collections::BTreeMap;

    const CASES:&str=include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/dated-cases.tsv"));
    const FLOWS:&str=include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/dated-flows.tsv"));
    const QUANTITIES:[&str;6]=["price derivative","price second derivative","time weighted amount",
        "modified duration","macaulay duration","convexity"];
    type Row<'a>=BTreeMap<&'a str,&'a str>;
    fn rows(text:&str)->Vec<Row<'_>> {
        let mut lines=text.lines();let header:Vec<_>=lines.next().expect("header").split('\t').collect();
        lines.map(|line|{let fields:Vec<_>=line.split('\t').collect();assert_eq!(fields.len(),header.len());header.iter().copied().zip(fields).collect()}).collect()
    }
    fn date(s:&str)->Date {
        let mut p=s.split('-');let y=number(p.next().expect("year"));let m=number(p.next().expect("month"));let d=number(p.next().expect("day"));
        assert!(p.next().is_none());Date::from_ymd(y,m,d).expect("transported valid civil date")
    }
    fn dates(s:&str)->Vec<Date>{s.split(',').map(date).collect()}
    fn geometry(actual:Geometry,expected:&str,label:&str){
        let c:Vec<_>=expected.split('|').collect();assert_eq!(c.len(),8,"{label}");
        assert_eq!(actual.start,date(c[0]),"{label}");assert_eq!(actual.end,date(c[1]),"{label}");
        assert_eq!(actual.full_count,number::<u32>(c[2]),"{label}");
        assert_eq!(actual.first_quasi,(date(c[3]),date(c[4])),"{label}");
        assert_eq!(actual.last_quasi,(date(c[5]),date(c[6])),"{label}");
        let fragments:Vec<_>=[actual.first,actual.second].into_iter().flatten().collect();
        let expected:Vec<_>=if c[7]=="-"{vec![]}else{c[7].split('~').collect()};
        assert_eq!(fragments.len(),expected.len(),"{label}");
        for (a,e) in fragments.iter().zip(expected){let e:Vec<_>=e.split(',').collect();
            assert_eq!(a.quasi_start,date(e[0]),"{label}");assert_eq!(a.quasi_end,date(e[1]),"{label}");
            assert_eq!(a.days,number::<u32>(e[2]),"{label}");assert_eq!(a.period_days,number::<u32>(e[3]),"{label}");}
    }
    fn instrument(c:&Row<'_>)->CanadianBond {
        let start=date(c["d_accrual_start"]);let maturity=date(c["d_stated_maturity"]);
        let anchor=date(c["d_anchor"]);let f=number::<u32>(c["d_frequency"]);
        let frequency=match f{2=>CanadianFrequency::Semiannual,4=>CanadianFrequency::Quarterly,12=>CanadianFrequency::Monthly,_=>panic!("frequency")};
        let eom=match c["d_end_of_month"]{"Ignore"=>EndOfMonth::Ignore,"Preserve"=>EndOfMonth::Preserve,_=>panic!("eom")};
        // Input reconstruction from the supplied anchor/f/EOM and D. Expected
        // dates are compared later and are never fed into this constructor.
        let mut quasi=vec![anchor];
        for k in 1..=20_000_u32 {
            let months=-i64::from(k*frequency.months());
            let mut q=anchor.checked_add_months(months).expect("input predecessor");
            if eom==EndOfMonth::Preserve && anchor.is_end_of_month(){q=q.last_day_of_month();}
            quasi.push(q);if q<=start{break;}
        }
        quasi.reverse();assert!(quasi[0]<=start);
        let first=date(c["d_first_coupon_date"]);let last_start=date(c["d_last_coupon_start_date"]);
        let mut coupons=vec![first];
        if first!=maturity {
            coupons.extend(quasi.iter().copied().filter(|d|*d>first&&*d<=last_start));
            coupons.push(maturity);
        }
        let schedule=CanadianCouponSchedule::new(start,maturity,frequency,quasi,coupons,eom,number(c["d_date_cap"])).expect("valid frozen input schedule");
        CanadianBond::new(date(c["d_original_issue"]),DecimalAmount::new(number(c["d_coupon_coefficient"]),number(c["d_coupon_scale"])).expect("coupon"),schedule,date(c["d_true_maturity"]),CanadianSourcePolicy::CanonicalIiac13LocalV1).expect("valid frozen input bond")
    }
    fn selection(c:&Row<'_>)->CanadianYieldSelection{
        match c["d_selection"]{"LastCouponMoneyMarket"=>CanadianYieldSelection::LastCouponMoneyMarket,
            "Explicit"=>CanadianYieldSelection::Explicit(match c["d_explicit_convention"]{"Compound"=>CanadianYieldConvention::Compound,"MoneyMarket"=>CanadianYieldConvention::MoneyMarket,_=>panic!("convention")}),_=>panic!("selection")}
    }
    fn check_risk(risk:&CanadianRisk,c:&Row<'_>,id:&str){
        let values=[risk.price_derivative(),risk.price_second_derivative(),risk.time_weighted_amount(),
            risk.modified_duration(),risk.macaulay_duration(),risk.convexity()];
        for (k,value) in values.into_iter().enumerate(){
            check_value(value,c[format!("e_view{k}_bits").as_str()],c[format!("e_view{k}_class").as_str()],QUANTITIES[k],QUANTITIES[k],None,id);
        }
    }
    fn check_prices(value:CanadianValuation,c:&Row<'_>,id:&str){
        let prices=[Ok(value.model_dirty_price()),value.clean_price(),value.settlement_dirty_price()];
        for (k,p) in prices.into_iter().enumerate(){check_price(p,c[format!("e_basis{k}_bits").as_str()],c[format!("e_basis{k}_class").as_str()],id);}
    }
    fn observed_seed(ordinal:usize,z:f64,bundles:&[Bundle],frozen_flows:&[&Row<'_>],factors:&[f64],calls:&mut[usize])->f64{
        let position=bundles.iter().position(|b|b.index==ordinal).expect("original ordinal");
        calls[position]+=1;
        assert_eq!(z.to_bits(),bits(frozen_flows[position]["exponent_bits"]));
        factors[position]
    }

    #[test]
    fn all_dated_frozen_inputs_bind_source_and_actual_traversal(){
        let all_flows=rows(FLOWS);let mut by_id:BTreeMap<&str,Vec<&Row<'_>>>=BTreeMap::new();
        for f in &all_flows {by_id.entry(f["id"]).or_default().push(f);}
        let mut counts=BTreeMap::new();let mut flow_count=0_usize;let mut exp_calls=0_usize;
        for c in rows(CASES){let id=c["id"];let op=c["operation"];*counts.entry(op).or_insert(0_usize)+=1;
            let bond=instrument(&c);let schedule=bond.schedule();
            assert_eq!(schedule.quasi_dates(),dates(c["e_quasi_dates"]),"{id}");
            assert_eq!(schedule.coupon_dates(),dates(c["e_actual_dates"]),"{id}");
            let plan=CanadianPricingPlan::new(&bond,date(c["d_settlement"]),selection(&c),number(c["d_max_bundles"])).expect(id);
            assert_eq!(plan.bundle_count(),by_id[id].len(),"{id}");
            assert_eq!(plan.bundles.iter().map(|b|b.index).collect::<Vec<_>>(),c["e_future_ordinals"].split(',').map(number::<usize>).collect::<Vec<_>>(),"{id}");
            assert_eq!(plan.metadata.yield_convention,if c["e_convention"]=="mm"{CanadianYieldConvention::MoneyMarket}else{CanadianYieldConvention::Compound},"{id}");
            assert_magnitude(&plan.valuation_ai.numerator,c["e_valuation_num"],id);assert_magnitude(&plan.valuation_ai.denominator,c["e_valuation_den"],id);
            assert_magnitude(&plan.settlement_ai.numerator,c["e_settlement_num"],id);assert_magnitude(&plan.settlement_ai.denominator,c["e_settlement_den"],id);
            assert_eq!(plan.valuation_accrued_per_100().to_bits(),bits(c["e_a_bits"]),"{id}");assert_eq!(plan.settlement_accrued_per_100().to_bits(),bits(c["e_ai_bits"]),"{id}");
            geometry(plan.accrued_geometry,c["e_accrued_geometry"],id);
            let frozen=&by_id[id];let factors:Vec<_>=if c["d_factor_bits"]=="-"{vec![]}else{c["d_factor_bits"].split(',').map(float).collect()};
            let y=float(c["d_yield_bits"]);let g=if c["d_g_bits"]=="-"{0.0}else{float(c["d_g_bits"])};
            let mut workspace=Workspace::new();let y=workspace.quote(plan.metadata,y).expect(id);
            let mut individual_calls=vec![0_usize;plan.bundle_count()];
            for (position,(bundle,flow)) in plan.bundles.iter().zip(frozen.iter()).enumerate(){flow_count+=1;
                assert_eq!(bundle.index,number::<usize>(flow["ordinal"]),"{id}");
                assert_eq!(schedule.coupon_dates()[bundle.index],date(flow["contractual_date"]),"{id}");
                assert_eq!(bond.coupon_payment_date(bundle.index),Some(date(flow["payment_date"])),"{id}");
                assert_eq!(bundle.days_to_payment,date(flow["payment_date"]).days_since(date(c["d_settlement"])).unsigned_abs(),"{id}");
                assert_eq!(bundle.days_payment_to_true,date(c["d_true_maturity"]).days_since(date(flow["payment_date"])).unsigned_abs(),"{id}");
                assert_magnitude(&bundle.amount.numerator,flow["source_num"],id);assert_magnitude(&bundle.amount.denominator,flow["source_den"],id);
                assert_eq!(bundle.lambda.numerator,number::<u64>(flow["lambda_num"]),"{id}");assert_eq!(bundle.lambda.denominator,number::<u32>(flow["lambda_den"]),"{id}");
                geometry(bond.coupon_geometry(bundle.index).expect(id),flow["coupon_geometry"],id);
                let factor=workspace.factor(plan.metadata,y,g,bundle,&mut |ordinal,z|observed_seed(ordinal,z,&plan.bundles,frozen,&factors,&mut individual_calls)).expect(id);
                assert_eq!(factor.to_bits(),bits(flow["factor_bits"]),"{id}: factor {position}");
                if op=="dated_mm" {assert_magnitude(&workspace.d,flow["affine_den"],id);assert_magnitude(&workspace.n,flow["affine_num"],id);}
                else {assert_eq!(workspace.logarithmic_exponent(g,plan.metadata.frequency.payments_per_year(),bundle.lambda,bundle.index).expect(id).to_bits(),bits(flow["exponent_bits"]),"{id}");}
                let pv=workspace.present_value(factor,bundle).expect(id);assert_eq!(pv.to_bits(),bits(flow["pv_bits"]),"{id}: PV {position}");
                let mut term_sums=[[0;243];3];workspace.risk_terms(plan.metadata,y,bundle,pv,&mut term_sums).expect(id);
                for k in 0..3 {let expected=flow[format!("retained{k}").as_str()];assert_magnitude(&term_sums[k],expected.strip_prefix('-').unwrap_or(expected),id);}
            }
            let mut sums=[[0;243];3];let mut terms=Vec::new();let mut calls=vec![0_usize;plan.bundle_count()];
            let dirty=workspace.traverse(&plan,y,g,&mut terms,Some(&mut sums),&mut |ordinal,z|observed_seed(ordinal,z,&plan.bundles,frozen,&factors,&mut calls)).expect(id);
            assert_eq!(dirty.to_bits(),bits(c["e_dirty_bits"]),"{id}");
            assert_eq!(terms.iter().map(|x|x.to_bits()).collect::<Vec<_>>(),frozen.iter().map(|f|bits(f["pv_bits"])).collect::<Vec<_>>(),"{id}");
            assert_eq!(calls,individual_calls,"{id}");
            for (position,flow) in frozen.iter().enumerate(){
                let expected=usize::from(op=="dated_compound_seeded" && bits(flow["exponent_bits"])&!SIGN!=0);
                assert_eq!(calls[position],expected,"{id}: exp count {position}");exp_calls+=calls[position];
            }
            for k in 0..3 {let expected=c[format!("e_retained{k}").as_str()];assert_magnitude(&sums[k],expected.strip_prefix('-').unwrap_or(expected),id);}
            let value=CanadianValuation{metadata:plan.metadata,yield_rate:y,dirty};check_prices(value,&c,id);
            let risk=CanadianRisk{metadata:plan.metadata,yield_rate:y,dirty,sums};check_risk(&risk,&c,id);
            if op=="dated_mm" {
                let public=plan.evaluate(y).expect(id);assert_eq!(public.model_dirty_per_100().to_bits(),dirty.to_bits(),"{id}");check_prices(public,&c,id);
                let public_risk=plan.risk(y).expect(id);check_risk(&public_risk,&c,id);
            }
        }
        assert_eq!(counts.get("dated_mm"),Some(&782));assert_eq!(counts.get("dated_compound_seeded"),Some(&437));assert_eq!(flow_count,3388);
        println!("ACTUAL_DATED_N_CASES {:?}; FLOW_STAGE_BINDINGS {}; SEEDED_EXP_CALLS {}",counts,flow_count,exp_calls);
    }
}
