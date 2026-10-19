---
id: mathematical-foundations-bayesian-inference-and-information
title: Conditional probability, Bayesian inference, information, and model selection
type: learning-module
status: active
created: 2026-08-15
updated: 2026-08-15
tags: [bayes, conditional-probability, information-theory, model-selection]
confidence: high
---

# Conditional probability, Bayesian inference, information, and model selection

Back to the [packet index](mathematical_foundations_index.md). Conditioning
updates a distribution after observation. Bayesian inference makes the prior,
data likelihood, and posterior distinct; information measures uncertainty on a
logarithmic scale.

$$
p(\theta\mid D)=\frac{p(D\mid\theta)p(\theta)}{p(D)}.
$$

## MF-04-01 — Original problem

A classifier marks 90% of positives and 5% of negatives positive. Prevalence
is 1%. Find $\mathbb P(\text{positive class}\mid\text{marked})$.

### Worked solution {#mf-04-01-solution}

Bayes’ rule gives $.9\cdot.01/(.9\cdot.01+.05\cdot.99)=.009/.0585$, about
$0.154$. A high sensitivity does not overcome a rare base rate by itself.

## MF-04-02 — Original problem

With a Beta$(2,2)$ prior for a coin probability and three heads, one tail,
what is the posterior?

### Worked solution {#mf-04-02-solution}

Beta-Bernoulli conjugacy adds successes and failures to its parameters. The
posterior is Beta$(5,3)$, with mean $5/8$.

## MF-04-03 — Original problem

Why can a MAP estimate differ from a maximum-likelihood estimate?

### Worked solution {#mf-04-03-solution}

Maximum likelihood maximizes $p(D\mid\theta)$; MAP maximizes
$p(D\mid\theta)p(\theta)$. A nonuniform prior shifts the optimum toward
parameter values judged plausible before observing data.

## MF-04-04 — Original problem

A fair binary variable has entropy in bits equal to what value?

### Worked solution {#mf-04-04-solution}

$H=-\sum_{x\in\{0,1\}}\tfrac12\log_2\tfrac12=1$ bit. It is maximal
because neither outcome is more predictable than the other.

## MF-04-05 — Original problem

If $X$ and $Y$ are independent, what is their mutual information?

### Worked solution {#mf-04-05-solution}

It is zero: $p(x,y)=p(x)p(y)$, making every logarithmic likelihood ratio
inside the expectation $\log[p(x,y)/(p(x)p(y))]$ equal to zero.

## MF-04-06 — Original problem

Two models have identical training error. One uses 2 adjustable coefficients
and one uses 200. Give one reason to prefer the smaller model before testing.

### Worked solution {#mf-04-06-solution}

It has fewer ways to fit accidental variation, so its expected generalization
error can be lower. This is a model-selection preference, not proof that it
will win on every future sample.

## MF-04-07 — Original problem

What is the predictive distribution conceptually obtained from a Bayesian
posterior $p(\theta\mid D)$?

### Worked solution {#mf-04-07-solution}

Average each parameter-specific prediction over posterior uncertainty:
$p(y_*\mid x_*,D)=\int p(y_*\mid x_*,\theta)p(\theta\mid D)d\theta$.
It includes parameter uncertainty rather than committing to one point estimate.

## MF-04-08 — Original problem

Why is negative log likelihood a natural loss for independent observations?

### Worked solution {#mf-04-08-solution}

Independence multiplies likelihoods, while a logarithm converts the product to
a sum: $-\log\prod_i p(y_i\mid x_i)=-\sum_i\log p(y_i\mid x_i)$. Additive
losses are easier to optimize and attribute per example.
